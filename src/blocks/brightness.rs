use super::{GetMarkup, GetName, IntoSerialized};
use rs_blocks_macros::{GetName, IntoSerialized, NoMarkup};
use serde::Deserialize;

#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Brightness {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_update_signal")]
	update_signal: i32,
	#[serde(default = "default_path_to_current_brightness")]
	path_to_current_brightness: String,
	#[serde(default = "default_path_to_max_brightness")]
	path_to_max_brightness: String,
}

fn default_period() -> u64 {
	600
}

fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR1
}

fn default_path_to_current_brightness() -> String {
	"/sys/class/backlight/intel_backlight/brightness".to_string()
}

fn default_path_to_max_brightness() -> String {
	"/sys/class/backlight/intel_backlight/max_brightness".to_string()
}
