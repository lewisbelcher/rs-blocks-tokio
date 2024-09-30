use super::{GetMarkup, GetName, IntoSerialized};
use rs_blocks_macros::GetName;
use serde::Deserialize;

// Add a derive macro with customisable defaults for name and period etc. Or separate derives for
// default name, default period etc?
#[derive(Debug, Deserialize, GetName)]
pub struct Battery {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
	#[serde(default = "default_path_to_charge_now")]
	path_to_charge_now: String,
	#[serde(default = "default_path_to_charge_full")]
	path_to_charge_full: String,
	#[serde(default = "default_path_to_status")]
	path_to_status: String,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.8
}

fn default_path_to_charge_now() -> String {
	"/sys/class/power_supply/BAT0/charge_now".to_string()
}

fn default_path_to_charge_full() -> String {
	"/sys/class/power_supply/BAT0/charge_full".to_string()
}

fn default_path_to_status() -> String {
	"/sys/class/power_supply/BAT0/status".to_string()
}

impl GetMarkup for Battery {}

impl IntoSerialized for Battery {
	fn get_full_text(&self) -> Option<String> {
		Some("Hello!".to_string())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::config;
	use crate::Block;

	#[test]
	fn configuration() {
		let string = "
			[Battery]
			period = 300
			alpha = 0.1
		";
		let mut deserialised = config::deserialise(string).unwrap();
		if let Block::Battery(battery) = deserialised.remove(0) {
			assert_eq!(battery.period, 300);
			assert_eq!(battery.alpha, 0.1);
		} else {
			panic!()
		};
	}
}
