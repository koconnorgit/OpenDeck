//! Event-driven active window tracking for KWin on Wayland.
//!
//! Polling the active window on KWin Wayland means loading, running and unloading a KWin script
//! on every poll, which blocks the compositor's main thread each time and causes a system-wide
//! stutter. Instead, a single persistent KWin script is loaded once; it reports the resource
//! class of the active window over D-Bus whenever focus changes.

use std::sync::OnceLock;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use zbus::Connection;

const KWIN_SERVICE: &str = "org.kde.KWin";
const SCRIPT_NAME: &str = "opendeck-active-window";
const OBJECT_PATH: &str = "/me/amankhanna/opendeck/ActiveWindow";
const INTERFACE: &str = "me.amankhanna.opendeck.ActiveWindow";
const RETRY_INTERVAL: Duration = Duration::from_secs(30);

static CONNECTION: OnceLock<Connection> = OnceLock::new();

struct ActiveWindow(UnboundedSender<String>);

#[zbus::interface(name = "me.amankhanna.opendeck.ActiveWindow")]
impl ActiveWindow {
	fn changed(&self, class: String) {
		let _ = self.0.send(class);
	}
}

async fn call_scripting<B>(connection: &Connection, method: &str, body: &B) -> zbus::Result<zbus::Message>
where
	B: serde::Serialize + zbus::zvariant::DynamicType,
{
	connection.call_method(Some(KWIN_SERVICE), "/Scripting", Some("org.kde.kwin.Scripting"), method, body).await
}

async fn load_script(connection: &Connection) -> anyhow::Result<()> {
	let address = connection.unique_name().ok_or_else(|| anyhow::anyhow!("D-Bus connection has no unique name"))?;
	// `windowActivated`/`activeWindow` are the Plasma 6 names, `clientActivated`/`activeClient` the Plasma 5 ones.
	let script = format!(
		r#"function report(w) {{
	callDBus("{address}", "{OBJECT_PATH}", "{INTERFACE}", "Changed", w ? String(w.resourceClass) : "");
}}
(workspace.windowActivated || workspace.clientActivated).connect(report);
report(workspace.activeWindow || workspace.activeClient);
"#
	);

	// KWin reads the file itself, possibly after `run` has returned, so it is left in place.
	let path = std::env::var_os("XDG_RUNTIME_DIR")
		.map(std::path::PathBuf::from)
		.unwrap_or_else(std::env::temp_dir)
		.join("opendeck-kwin-active-window.js");
	tokio::fs::write(&path, script).await?;
	let path = path.to_str().ok_or_else(|| anyhow::anyhow!("script path is not valid UTF-8"))?;

	// A previous instance that didn't exit cleanly leaves its script behind, which blocks loading under the same name.
	let _ = call_scripting(connection, "unloadScript", &(SCRIPT_NAME,)).await;

	let id: i32 = call_scripting(connection, "loadScript", &(path, SCRIPT_NAME)).await?.body().deserialize()?;
	if id < 0 {
		anyhow::bail!("KWin refused to load the script");
	}

	// Plasma 6 exports scripts at /Scripting/Script<id>, Plasma 5 at /<id>.
	let mut result = Ok(());
	for object in [format!("/Scripting/Script{id}"), format!("/{id}")] {
		match connection.call_method(Some(KWIN_SERVICE), object, Some("org.kde.kwin.Script"), "run", &()).await {
			Ok(_) => return Ok(()),
			Err(error) => result = Err(error),
		}
	}
	let _ = call_scripting(connection, "unloadScript", &(SCRIPT_NAME,)).await;
	Ok(result?)
}

/// Start reporting the active window's resource class through `sender`.
///
/// Returns `false` if this is not a KWin Wayland session, in which case the caller should fall back to polling.
pub async fn start(sender: UnboundedSender<String>) -> bool {
	if std::env::var_os("WAYLAND_DISPLAY").is_none() {
		return false;
	}

	let Ok(builder) = zbus::connection::Builder::session().and_then(|builder| builder.serve_at(OBJECT_PATH, ActiveWindow(sender))) else {
		return false;
	};
	let Ok(connection) = builder.build().await else {
		return false;
	};
	let Ok(dbus) = zbus::fdo::DBusProxy::new(&connection).await else {
		return false;
	};
	let Ok(kwin) = zbus::names::BusName::try_from(KWIN_SERVICE) else {
		return false;
	};
	if !dbus.name_has_owner(kwin).await.unwrap_or(false) {
		return false;
	}

	let _ = CONNECTION.set(connection.clone());

	tokio::spawn(async move {
		// The script is lost when KWin restarts, so it is loaded again whenever the service gets a new owner.
		let mut owner_changes = dbus.receive_name_owner_changed_with_args(&[(0, KWIN_SERVICE)]).await.ok();
		loop {
			if let Err(error) = load_script(&connection).await {
				log::warn!("Failed to load KWin active window script: {error}");
				tokio::time::sleep(RETRY_INTERVAL).await;
				continue;
			}
			log::info!("Tracking the active window using a KWin script");

			let Some(owner_changes) = owner_changes.as_mut() else {
				return;
			};
			loop {
				let Some(signal) = owner_changes.next().await else {
					return;
				};
				if signal.args().is_ok_and(|args| args.new_owner().is_some()) {
					break;
				}
			}
		}
	});

	true
}

/// Unload the KWin script, if one was loaded.
pub async fn stop() {
	if let Some(connection) = CONNECTION.get() {
		let _ = tokio::time::timeout(Duration::from_secs(1), call_scripting(connection, "unloadScript", &(SCRIPT_NAME,))).await;
	}
}
