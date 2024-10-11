use super::prelude::*;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;
use signal_hook_tokio::Signals;
use std::fmt::{self, Display, Formatter};
use tokio::process::Command;

const AUDIO_DRIVER_COMMAND: &str = "pulsemixer";

#[with_fields(period)]
#[derive(Debug, Deserialize, NoMarkup, GetName, IntoSerialized)]
pub struct Volume {
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_period() -> u64 {
	2000
}

fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR2
}

#[derive(Debug, PartialEq)]
enum VolumeStatus {
	Mute,
	Value(u8),
}

impl TryFrom<String> for VolumeStatus {
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let mut lines = value.lines();

		let line1 = lines.next().ok_or_else(|| Error::Parse {
			origin: format!("{} mute indicator", AUDIO_DRIVER_COMMAND),
			ty: "string",
		})?;
		if line1.trim() == "1" {
			return Ok(VolumeStatus::Mute);
		}

		lines
			.next()
			.and_then(|line2| {
				line2
					.split_whitespace()
					.next()
					.and_then(|word1| word1.parse::<u8>().ok())
			})
			.ok_or_else(|| Error::Parse {
				origin: format!("{} volume value", AUDIO_DRIVER_COMMAND),
				ty: "u8",
			})
			.map(VolumeStatus::Value)
	}
}

impl Display for VolumeStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		let text = match self {
			VolumeStatus::Mute => "",
			VolumeStatus::Value(x) => &format!("   {x}%"),
		};
		write!(f, "{}", text)
	}
}

impl IntoStream for Volume {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let mut signals =
			Signals::new([self.update_signal]).expect("failed to initialise volume signal hook");
		let duration = std::time::Duration::from_millis(self.period);

		stream! {
			let mut command = Command::new(AUDIO_DRIVER_COMMAND);
			command.args(["--get-mute", "--get-volume"]);
			loop {
				// Ignore the Result, it's fine if the timeout elapses
				let _ = tokio::time::timeout(duration, signals.next()).await;
				// TODO: This parsing looks very similar to `util::read_to_ty`
				let status: VolumeStatus = command.output()
					.await
					.map(|x| String::from_utf8(x.stdout))
					.map_err(Error::Io)?
					.ok()
					.and_then(|x| x.try_into().ok())
					.ok_or_else(|| Error::Parse { origin: AUDIO_DRIVER_COMMAND.to_string(), ty: "UTF-8 string"})?;
				yield Ok(format!("{}", status));
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn into_volume_status() {
		let input = "0\n70 70\n".to_string();
		let status: VolumeStatus = input.try_into().unwrap();
		assert_eq!(status, VolumeStatus::Value(70));

		let input = "1\n70 70\n".to_string();
		let status: VolumeStatus = input.try_into().unwrap();
		assert_eq!(status, VolumeStatus::Mute);
	}
}
