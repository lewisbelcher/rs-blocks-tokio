use super::{default_period, prelude::*};
use rs_blocks_macros::*;
use serde::Deserialize;

#[with_fields(period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Volume {
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR2
}
