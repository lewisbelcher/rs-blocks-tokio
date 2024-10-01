use super::{GetMarkup, GetName, IntoSerialized};
use rs_blocks_macros::{GetName, NoMarkup,IntoSerialized};
use serde::Deserialize;

#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Cpu {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.7
}
