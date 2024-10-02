use super::{GetMarkup, GetName, IntoSerialized, IntoStream};
use crate::blocks::util;
use crate::Error;
use async_stream::stream;
use futures_util::{pin_mut, Stream, StreamExt};
use rs_blocks_macros::{GetName, IntoSerialized, NoMarkup};
use serde::Deserialize;
use std::cell::OnceCell;

const PATTERN: &str = r"(?s)MemTotal:\s+(\d+).+MemFree:\s+(\d+)";
const CELL: OnceCell<regex::Regex> = OnceCell::new();

#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Memory {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
	#[serde(default = "default_meminfo_path")]
	meminfo_path: String,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.7
}

fn default_meminfo_path() -> String {
	"/proc/meminfo".to_string()
}

#[derive(Debug, PartialEq)]
struct MemStats {
	total: f32,
	free: f32,
}

impl MemStats {
	fn percent(&self) -> f32 {
		100.0 * (1.0 - self.free / self.total)
	}
}

impl TryFrom<regex::Captures<'_>> for MemStats {
	type Error = Error;

	fn try_from(captures: regex::Captures<'_>) -> Result<Self, Self::Error> {
		// TODO: What would we do if we wanted to specify the file path in the error?
		Ok(MemStats {
			total: extract_match(captures.get(1), "Memory")?,
			free: extract_match(captures.get(2), "Memory")?,
		})
	}
}

fn extract_match(m: Option<regex::Match>, name: &'static str) -> Result<f32, Error> {
	m.ok_or_else(|| Error::PatternMatch { name })?
		.as_str()
		.parse()
		.map_err(|_| Error::PatternMatch { name })
}

impl IntoStream for Memory {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let re = regex::Regex::new(PATTERN).unwrap();
		stream! {
			let mut watcher = Box::pin(util::watch(&self.meminfo_path, self.period));
			let mut ema = util::Ema::new(self.alpha);
			while let Some(contents) = watcher.next().await {
				let mem_stats: MemStats = re.captures(&contents?)
					.ok_or_else(|| Error::PatternMatch { name: Self::get_name() })?
					.try_into()?;
				ema.push(mem_stats.percent());
				yield Ok(format!(" {:.1}%", ema));
			}
		}
	}
}
