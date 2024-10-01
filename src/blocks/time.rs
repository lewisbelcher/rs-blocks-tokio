use super::{GetMarkup, GetName, IntoSerialized};
use rs_blocks_macros::{GetName, IntoSerialized, PangoMarkup};
use serde::Deserialize;

#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Time {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_format")]
	format: String,
}

fn default_period() -> u64 {
	1000
}

fn default_format() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}
