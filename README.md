# OpenDeck

Linux software for your Elgato Stream Deck

![Main menu](.github/readme/mainmenu.png)
[More screenshots](#showcase)

> [!NOTE]
> This is [**koconnorgit/OpenDeck**](https://github.com/koconnorgit/OpenDeck), a personal fork of upstream [**nekename/OpenDeck**](https://github.com/nekename/OpenDeck). It carries in-progress fixes for the **Stream Deck + XL** (encoder events, rotated-mount icon defaults, 100×100 LCD region-write constraints, updated udev rules) that have not yet been upstreamed. For official releases and general support, go to the upstream project.

OpenDeck is a desktop application for using stream controller devices like the Elgato Stream Deck on Linux, Windows, and macOS. OpenDeck supports plugins made for the original Stream Deck SDK, allowing many plugins made for the Elgato software ecosystem to be used, or the [OpenAction](https://openaction.amankhanna.me/) API.

Only Elgato hardware is officially supported, but plugins are available for support for other hardware vendors.

> [!TIP]
> No Stream Deck in front of you? Use OpenDeck with [Tacto](https://tacto.live/) to turn any smartphone into one!

If you would like to support development of OpenDeck, consider [sponsoring me](https://github.com/sponsors/nekename) on GitHub Sponsors! Considering that the power of your Stream Deck comes from the software you use with it, just $5 (only 2.5% of the cost of a Stream Deck+) goes a long way.

Special thanks go to the developers of [Tauri](https://github.com/tauri-apps/tauri), the [elgato-streamdeck](https://github.com/OpenActionAPI/rust-elgato-streamdeck) Rust library, and [Phosphor Icons](https://phosphoricons.com/).

### Why use OpenDeck?

- **Stream Deck plugins**: OpenDeck supports the majority of the Stream Deck plugins that users of the Elgato ecosystem are already familiar with, unlike other third-party softwares which are much more limited (e.g. streamdeck-ui, StreamController, Boatswain etc).
- **Cross-platform**: OpenDeck supports Linux alongside Windows and macOS. macOS users also benefit from switching from the first-party Elgato software as OpenDeck can run plugins only built for Windows on Linux and macOS thanks to Wine. Additionally, profile files are easily moveable between platforms with no changes to them necessary.
- **Feature-packed**: From Multi Actions and Toggle Actions to switching profiles when you switch apps and brightness control, OpenDeck has all the features you'd expect from stream controller software.
- **Open source**: OpenDeck source code is licensed under the GNU General Public License, allowing anyone to view it and improve it for feature, stability, privacy or security reasons. [Most plugins are open-source, too.](https://marketplace.rivul.us/)
- **Written in Rust**: The Rust programming language, which OpenDeck is built with alongside TypeScript, is known for its performance, safety and resulting code quality.

## Installation

### Linux

This fork ships no prebuilt release artifacts yet — installation means **building the AppImage from source**. The recipe below is tested on CachyOS and should work on other Arch-based rolling distributions with minor adjustment.

#### 1. Prerequisites

- All [Tauri build prerequisites](https://tauri.app/start/prerequisites) for your distribution (Rust toolchain, WebKitGTK, glib, etc.).
- **Node.js** and **npm** (the `tauri` CLI is invoked via the `package.json` scripts).
- **libudev** development headers (`libudev` / `systemd-libs` depending on distro — needed by hidapi).
- **Wine** (optional): required only if you want to run Stream Deck plugins that only ship Windows binaries. Some plugins also depend on Wine Mono.

#### 2. Clone and build

```sh
git clone https://github.com/koconnorgit/OpenDeck.git ~/.opendeck/OpenDeck
cd ~/.opendeck/OpenDeck
npm install
NO_STRIP=1 npm run tauri build
```

> [!IMPORTANT]
> `NO_STRIP=1` is required on CachyOS and other rolling-release distros that ship modern system libraries with `.relr.dyn` ELF sections (e.g. `libwebp.so`, `libzstd.so`). The `strip` binary bundled inside `linuxdeploy` (cached at `~/.cache/tauri/linuxdeploy-x86_64.AppImage`) is too old to parse those sections, and without the flag the AppImage bundle step fails with `unknown type [0x13] section .relr.dyn`. Binaries end up slightly larger (debug info retained) but are fully functional.

#### 3. Install the AppImage

```sh
install -D src-tauri/target/release/bundle/appimage/opendeck_*.AppImage ~/.local/bin/OpenDeck.AppImage
```

The AppImage reads config from `~/.config/opendeck/`, so plugins and profiles carry over from any prior install.

#### 4. Install the udev subsystem rules

Stream Decks on Linux are accessed via `/dev/hidraw*`, which requires a udev rule granting your user `uaccess`. Install the bundled rules file and reload:

```sh
sudo install -m 0644 src-tauri/bundle/40-streamdeck.rules /etc/udev/rules.d/40-streamdeck.rules
sudo udevadm control --reload
```

Then **physically unplug and replug** each Stream Deck. `udevadm trigger` alone does not reliably reapply `uaccess` to an already-connected device — a real replug is the only thing that consistently does.

Verify each Stream Deck's hidraw node now has your user in the ACL:

```sh
for h in /dev/hidraw*; do
	udevadm info --query=property --name="$h" 2>/dev/null | grep -q 'ID_VENDOR_ID=0fd9' || continue
	echo "=== $h ==="; getfacl "$h" | grep -E '^user:'
done
```

You should see a `user:<you>:rw-` line for each connected Stream Deck. If not, see [Troubleshooting](#troubleshooting) below.

#### 5. Run it

```sh
~/.local/bin/OpenDeck.AppImage
```

To autostart OpenDeck hidden in the system tray on login, create `~/.config/autostart/opendeck.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=OpenDeck
Exec=/home/YOUR_USERNAME/.local/bin/OpenDeck.AppImage --hide
X-GNOME-Autostart-enabled=true
```

### Windows

- Download the latest release (`.exe` or `.msi`) from [GitHub Releases](https://github.com/nekename/OpenDeck/releases/latest).
- Double-click the downloaded file to run the installer.

### macOS

- Download the latest release from [GitHub Releases](https://github.com/nekename/OpenDeck/releases/latest).
- If you downloaded a `.dmg`, open the downloaded disk image and drag the application inside into your Applications folder; otherwise, extract the `.tar.gz` to your Applications folder.
- Open the installed application. Note: if you receive a warning about OpenDeck being distributed by an unknown developer, *right-click the app in Finder and then click Open* to suppress the warning.
- If you intend to use plugins that are only compiled for Windows, you will need to have [Wine](https://www.winehq.org/) installed on your system.

## Support

### How do I...?

To edit an action's settings, left-click on it to display its *property inspector*. To remove an action, right-click on it and choose "Delete" from the context menu.

To edit an action's appearance, right-click on it and select "Edit" from the context menu. You can then customise the image and text for each of its states. Left-click on the image to choose an image from your filesystem or right-click on the image to reset it to the plugin-provided default.

To select another device, or to switch profiles, use the dropdowns in the top right corner. You can organise profiles into folders by prefixing the profile name with the folder name and a forward slash. You can also configure automatically switching to a profile when a specific application's window is active.

To change other options, open Settings. From here, you can also view information about your version of OpenDeck or open the configuration and log directories. To add or remove plugins, visit the Plugins tab.

### Troubleshooting

- Ensure you are running the latest version of OpenDeck, as well as recent versions of related software (e.g. Spotify or OBS).
- Check the [FAQ](https://github.com/nekename/OpenDeck/wiki/0.-FAQ) and [GitHub Issues](https://github.com/nekename/OpenDeck/issues) to see if there's a fix for your problem already.
- Check the OpenDeck log file for any important messages. This file should be included with any support request.
	- You can also run OpenDeck from the terminal to see the logs directly if it's easier than finding the log file or if the log file is empty or missing details.
	- For issues with plugins, you can also check the plugin's logs (in the same folder, sometimes as well as a file named `plugin.log` or similar in the plugin's own folder).
	- The log directory can be opened from the settings page of OpenDeck, or alternatively located manually at the paths below:
		- Linux: `~/.local/share/opendeck/logs/`
		- Flatpak: `~/.var/app/me.amankhanna.opendeck/data/opendeck/logs/`
		- Windows: `%appdata%\opendeck\logs\`
		- macOS: `~/Library/Logs/opendeck/`
- When trying to run compiled plugins built for Windows on Linux or macOS, please ensure you have the latest version of Wine (and Wine Mono) installed on your system.
- If your device isn't showing up, ensure you have the correct permissions to access it (e.g. on Linux, installing udev subsystem rules and restarting your system), and that you have restarted OpenDeck since connecting it.
	- On Linux, OpenDeck talks to Stream Decks via `/dev/hidraw*`. If a specific device works on one machine but not another — or one of two connected Stream Decks is missing — an older `40-streamdeck.rules` / `60-streamdeck.rules` file on disk is the most common cause. Newer Stream Deck models (including the Stream Deck + XL, `0fd9:00c6`) may not be listed.
	- To diagnose: find the device's hidraw node with `for h in /dev/hidraw*; do udevadm info --query=property --name="$h" | grep -q '<PID>' && echo "$h"; done` (replacing `<PID>` with the product ID reported by `lsusb | grep Elgato`), then `getfacl <node>`. If there's no `user:<you>:rw-` line, udev isn't granting the ACL.
	- To fix: make sure the installed rules file contains **both** a `SUBSYSTEM=="usb"` line *and* a `KERNEL=="hidraw*"` line for your device's product ID. A USB-only rule is not enough — hidapi on Linux uses `/dev/hidraw*`, and the hidraw child does not inherit `uaccess` from the parent USB device. The canonical rules file is [`src-tauri/bundle/40-streamdeck.rules`](src-tauri/bundle/40-streamdeck.rules) in this repo. After editing, `sudo udevadm control --reload` and **physically unplug/replug** the device — `udevadm trigger` alone does not reliably reapply `uaccess` to an already-connected device. Then fully quit and relaunch OpenDeck; it does not retry failed device opens automatically.

### Support forums

- [Discord](https://discord.gg/26Nf8rHvaj)
- [Matrix](https://matrix.to/#/#opendeck:matrix.org)
- [GitHub Issues](https://github.com/nekename/OpenDeck/issues)

### Building from source / contributing

> [!TIP]
> The development guide for agents present in [AGENTS.md](AGENTS.md) also serves as a useful introduction to the codebase for humans.

You'll need to ensure that all of the [prerequisites for building a Tauri application](https://tauri.app/start/prerequisites) are satisfied to build OpenDeck, as well as making sure that [Deno](https://deno.com/) is installed. On Linux, you'll also need `libudev` installed for your distribution. After running `deno install`, you can use `deno task tauri dev` and `deno task tauri build` to work with OpenDeck.

Before each commit, please ensure that all of the following are completed:
1. Rust code has been linted using `cargo clippy` and it discovers no violations
2. Rust code has been formatted using `cargo fmt`
3. TypeScript code has been checked using `deno check` and linted using `deno lint` and they discover no violations
4. Svelte code has been linted using `deno task check` and it discovers no violations
5. Frontend code has been formatted using `deno fmt --unstable-component`

When submitting contributions, please adhere to the [Conventional Commits specification](https://conventionalcommits.org/) for commit messages. You will also need to [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits). Feel free to reach out on the support channels above for guidance when contributing!

OpenDeck is licensed under the GNU General Public License version 3.0 or later. For more details, see the LICENSE.md file.

## Showcase

![Main menu](.github/readme/mainmenu.png)
![Multi action](.github/readme/multiaction.png)
![Plugins](.github/readme/plugins.png)
![Profiles](.github/readme/profiles.png)
