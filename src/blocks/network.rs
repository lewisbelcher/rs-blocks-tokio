use super::{GetMarkup, GetName, IntoSerialized};
use rs_blocks_macros::{GetName, IntoSerialized, PangoMarkup};
use serde::Deserialize;

#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Network {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
	path_to_rx: String,
	path_to_tx: String,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.7
}
