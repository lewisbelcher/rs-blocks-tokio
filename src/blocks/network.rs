use super::{default_alpha, default_period, prelude::*};
use rs_blocks_macros::*;
use serde::Deserialize;

#[with_fields(alpha, period)]
#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Network {
	path_to_rx: String,
	path_to_tx: String,
}
