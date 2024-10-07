use super::{default_alpha, default_period, prelude::*};
use crate::blocks::util;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;

const PATTERN: &str = r"(?x)
cpu\s+
(?<user>\d+)\s+
(?<nice>\d+)\s+
(?<system>\d+)\s+
(?<idle>\d+)\s+
(?<iowait>\d+)\s+
(?<irq>\d+)\s+
(?<softirq>\d+)\s+
(?<steal>\d+)";

#[with_fields(alpha, period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Cpu {
	#[serde(default = "default_cpu_stat_path")]
	cpu_stat_path: String,
}

fn default_cpu_stat_path() -> String {
	"/proc/stat".to_string()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CpuStats {
	idle: f32,
	total: f32,
}

impl CpuStats {
	fn percent(&self, prev: Self) -> Option<f32> {
		if self.total != prev.total {
			Some((1.0 - (self.idle - prev.idle) / (self.total - prev.total)) * 100.0)
		} else {
			None
		}
	}
}

impl TryFrom<regex::Captures<'_>> for CpuStats {
	type Error = Error;

	fn try_from(captures: regex::Captures<'_>) -> Result<Self, Self::Error> {
		// TODO: What would we do if we wanted to specify the file path in the error?
		let user = extract_match(captures.name("user"))?;
		let nice = extract_match(captures.name("nice"))?;
		let system = extract_match(captures.name("system"))?;
		let idle = extract_match(captures.name("idle"))?;
		let iowait = extract_match(captures.name("iowait"))?;
		let irq = extract_match(captures.name("irq"))?;
		let softirq = extract_match(captures.name("softirq"))?;
		let steal = extract_match(captures.name("steal"))?;
		let idle = idle + iowait;
		let total = user + nice + system + idle + iowait + irq + softirq + steal;
		Ok(CpuStats { idle, total })
	}
}

fn extract_match(m: Option<regex::Match>) -> Result<f32, Error> {
	m.ok_or_else(|| Error::PatternMatch { name: "Cpu" })?
		.as_str()
		.parse()
		.map_err(|_| Error::PatternMatch { name: "Cpu" })
}

impl IntoStream for Cpu {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let re = regex::Regex::new(PATTERN).unwrap();
		stream! {
			let mut watcher = Box::pin(util::watch(&self.cpu_stat_path, self.period));
			let mut ema = util::Ema::new(self.alpha);
			let mut prev = None;
			while let Some(contents) = watcher.next().await {
				let cpu_stats: CpuStats = re.captures(&contents?)
					.ok_or_else(|| Error::PatternMatch { name: Self::get_name() })?
					.try_into()?;
				if let Some(prev) = prev.replace(cpu_stats) {
					if let Some(percent) = cpu_stats.percent(prev) {
						ema.push(percent);
						yield Ok(format!(" {:.1}%", ema));
					}
				}
			}
		}
	}
}
