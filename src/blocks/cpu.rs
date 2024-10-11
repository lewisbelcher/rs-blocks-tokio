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

#[derive(Clone, Copy, Debug, PartialEq, TryFromCaptures)]
struct CpuStats {
	user: f32,
	nice: f32,
	system: f32,
	idle: f32,
	iowait: f32,
	irq: f32,
	softirq: f32,
	steal: f32,
}

impl CpuStats {
	fn percent(&self, prev: Self) -> Option<f32> {
		let idle = self.idle + self.iowait;
		let total = self.total();
		let prev_total = prev.total();
		if total != prev_total {
			let prev_idle = prev.idle + prev.iowait;
			Some((1.0 - (idle - prev_idle) / (total - prev_total)) * 100.0)
		} else {
			None
		}
	}

	fn total(&self) -> f32 {
		self.user
			+ self.nice
			+ self.system
			+ self.idle
			+ self.iowait
			+ self.irq
			+ self.softirq
			+ self.steal
	}
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
					.and_then(|x| x.try_into().ok())
					.ok_or_else(|| Error::Parse { origin: self.cpu_stat_path.to_string(), ty: "CpuStats" })?;
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
