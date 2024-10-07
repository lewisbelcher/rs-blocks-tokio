use super::prelude::*;
use crate::Error;
use async_stream::stream;
use chrono::prelude::*;
use futures_util::Stream;
use rs_blocks_macros::*;
use serde::Deserialize;
use tokio::time::{self, Duration};

#[with_fields(period)]
#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Time {
	#[serde(default = "default_format")]
	format: String,
}

fn default_period() -> u64 {
	1000
}

fn default_format() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}

impl IntoStream for Time {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		stream! {
			let mut interval = time::interval(Duration::from_millis(self.period));
			loop {
				yield Ok(Local::now().format(&self.format).to_string());
				interval.tick().await;
			}
		}
	}
}
