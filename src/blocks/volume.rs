use super::prelude::*;
use crate::blocks::util;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;
use signal_hook_tokio::Signals;
use std::fmt::{self, Display, Formatter};
use tokio::process::Command;

const AUDIO_DRIVER_COMMAND: &str = "pulsemixer";
const PATTERN: &str = r"(?<mute>\d)\n(?<level>\d+)";

#[with_fields(period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Volume {
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_period() -> u64 {
	2000
}

// TODO: Use tokio::signal::unix::SignalKind
fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR2
}

#[derive(TryFromCaptures)]
struct VolumeStats {
	mute: u8,
	level: u8,
}

impl Display for VolumeStats {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		let text = if self.mute == 1 {
			""
		} else {
			&format!("   {}%", self.level)
		};
		write!(f, "{}", text)
	}
}

impl IntoStream for Volume {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let mut signals =
			Signals::new([self.update_signal]).expect("failed to initialise volume signal hook");
		let duration = std::time::Duration::from_millis(self.period);
		let re = regex::Regex::new(PATTERN).unwrap();
		let mut command = Command::new(AUDIO_DRIVER_COMMAND);
		command.args(["--get-mute", "--get-volume"]);

		stream! {
			loop {
				// Ignore the Result, it's fine if the timeout elapses
				let _ = tokio::time::timeout(duration, signals.next()).await;
				let contents = command.output()
					.await
					.map(|x| String::from_utf8(x.stdout))
					.map_err(Error::Io)?
					.map_err(|_| Error::Parse {
						name: Self::get_name(),
						reason: "couldn't convert stdout to UTF-8 string".to_string()
					})?;
				let stats: VolumeStats = util::from_string(&re, &contents, Self::get_name())?;
				yield Ok(format!("{}", stats));
			}
		}
	}
}
