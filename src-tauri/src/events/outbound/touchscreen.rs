use super::{Coordinates, send_to_plugin};

use crate::shared::ActionContext;
use crate::store::profiles::{acquire_locks_mut, get_instance_mut};

use serde::Serialize;

#[derive(Serialize)]
#[allow(non_snake_case)]
struct TouchTapPayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	controller: &'static str,
	tapPos: [u16; 2],
	hold: bool,
}

#[derive(Serialize)]
struct TouchTapEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: TouchTapPayload,
}

/// Each encoder zone on both Stream Deck + and + XL is 200px wide on the LCD strip.
const ENCODER_ZONE_WIDTH: u16 = 200;

pub async fn touch_tap(device: &str, x: u16, y: u16, hold: bool) -> Result<(), anyhow::Error> {
	let index = (x / ENCODER_ZONE_WIDTH) as u8;

	let mut locks = acquire_locks_mut().await;
	let selected_profile = locks.device_stores.get_selected_profile(device)?;
	let context = ActionContext {
		device: device.to_owned(),
		profile: selected_profile.to_owned(),
		controller: "Encoder".to_owned(),
		position: index,
		index: 0,
	};
	let Some(instance) = get_instance_mut(&context, &mut locks).await? else { return Ok(()) };

	send_to_plugin(
		&instance.action.plugin,
		&TouchTapEvent {
			event: "touchTap",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: TouchTapPayload {
				settings: instance.settings.clone(),
				coordinates: Coordinates {
					row: 0,
					column: instance.context.position,
				},
				controller: "Encoder",
				tapPos: [x, y],
				hold,
			},
		},
	)
	.await
}
