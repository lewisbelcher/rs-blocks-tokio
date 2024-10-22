use crate::blocks::{default_alpha, default_period, prelude::*, util};
use crate::Error;
use async_stream::stream;
use futures_util::Stream;
use rs_blocks_macros::*;
use serde::Deserialize;

const PATTERN: &str = r"(?s)MemTotal:\s+(?<total>\d+).+MemFree:\s+(?<free>\d+)";

#[with_fields(alpha, period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Memory {
	#[serde(default = "default_meminfo_path")]
	meminfo_path: String,
}

fn default_meminfo_path() -> String {
	"/proc/meminfo".to_string()
}

#[derive(TryFromCaptures)]
struct MemStats {
	total: f32,
	free: f32,
}

impl MemStats {
	fn percent(&self) -> f32 {
		100.0 * (1.0 - self.free / self.total)
	}
}

impl IntoStream for Memory {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let re = regex::Regex::new(PATTERN).unwrap();
		let mut ema = util::Ema::new(self.alpha);
		stream! {
			let watcher = util::watch(&self.meminfo_path, self.period);
			for await contents in watcher {
				let stats: MemStats = util::from_string(&re, &contents?, Self::get_name())?;
				ema.push(stats.percent());
				yield Ok(format!(" {:.1}%", ema));
			}
		}
	}
}
