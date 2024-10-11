use super::prelude::*;
use crate::blocks::util;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;
use signal_hook_tokio::Signals;

#[with_fields(period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Brightness {
	#[serde(default = "default_update_signal")]
	update_signal: i32,
	#[serde(default = "default_path_to_current_brightness")]
	path_to_current_brightness: String,
	#[serde(default = "default_max_brightness")]
	max_brightness: u32,
}

fn default_period() -> u64 {
	2000
}

fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR1
}

fn default_path_to_current_brightness() -> String {
	"/sys/class/backlight/intel_backlight/brightness".to_string()
}

fn default_max_brightness() -> u32 {
	120000
}

impl IntoStream for Brightness {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let mut signals = Signals::new([self.update_signal])
			.expect("failed to initialise brightness signal hook");
		let duration = std::time::Duration::from_millis(self.period);
		let max_brightness = self.max_brightness / 100; // Adjust for getting the percentage

		stream! {
			loop {
				// Ignore the Result, it's fine if the timeout elapses
				let _ = tokio::time::timeout(duration, signals.next()).await;
				let current: u32 = util::read_to_ty(Self::get_name(), &self.path_to_current_brightness).await?;
				yield Ok(format!(" {:.0}%", current / max_brightness));
			}
		}
	}
}
