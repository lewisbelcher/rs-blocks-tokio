use super::prelude::*;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;
use signal_hook_tokio::Signals;
use std::fmt::{self, Display, Formatter};
use tokio::process::Command;

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
			name: "Volume",
			ty: "mute string",
		})?;
		if line1.trim() == "1" {
			return Ok(VolumeStatus::Mute);
		}
		let line2 = lines.next().ok_or_else(|| Error::Parse {
			name: "Volume",
			ty: "volume value",
		})?;
		let word1 = line2
			.trim()
			.split_whitespace()
			.next()
			.ok_or_else(|| Error::Parse {
				name: "Volume",
				ty: "volume value",
			})?;
		let num = word1.parse::<u8>().map_err(|_| Error::Parse {
			name: "Volume",
			ty: "volume value",
		})?;
		Ok(VolumeStatus::Value(num))
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
			Signals::new(&[self.update_signal]).expect("failed to initialise volume signal hook");
		let duration = std::time::Duration::from_millis(self.period);

		stream! {
			let mut command = Command::new("pulsemixer");
			command.args(&["--get-mute", "--get-volume"]);
			loop {
				// Ignore the Result, it's fine if the timeout elapses
				let _ = tokio::time::timeout(duration, signals.next()).await;
				let status: VolumeStatus = command.output()
					.await
					.map(|x| String::from_utf8(x.stdout))
					.map_err(Error::Io)?
					.ok()
					.and_then(|x| x .try_into().ok())
					.ok_or_else(|| Error::Parse { name: "Volume", ty: "UTF-8 string"})?;
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
