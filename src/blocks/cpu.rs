use crate::blocks::{default_alpha, default_period, prelude::*, util, StreamExt2};
use crate::Error;
use async_stream::try_stream;
use futures_util::Stream;
use rs_blocks_macros::*;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

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
		let mut ema = util::Ema::new(self.alpha);
		let mut prev = None;
		try_stream! {
			let watcher = util::watch::<_, 100>(&self.cpu_stat_path)
				.with_period(|| sleep(Duration::from_millis(self.period)));
			for await contents in watcher {
				let stats: CpuStats = util::from_string(&re, &contents?)?;
				if let Some(prev) = prev.replace(stats) {
					if let Some(percent) = stats.percent(prev) {
						ema.push(percent);
						yield format!("  {:.1}%", ema);
					}
				}
			}
		}
	}
}
