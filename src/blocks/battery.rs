use super::{default_period, prelude::*};
use crate::blocks::util;
use crate::Error;
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use rs_blocks_macros::*;
use serde::Deserialize;
use std::fmt;
use std::time::Instant;

// Add a derive macro with customisable defaults for name and period etc. Or separate derives for
// default name, default period etc?
#[with_fields(alpha, period)]
#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Battery {
	#[serde(default = "default_path_to_charge_now")]
	path_to_charge_now: String,
	#[serde(default = "default_path_to_charge_full")]
	path_to_charge_full: String,
	#[serde(default = "default_path_to_status")]
	path_to_status: String,
}

fn default_alpha() -> f32 {
	0.95
}

fn default_path_to_charge_now() -> String {
	"/sys/class/power_supply/BAT0/charge_now".to_string()
}

fn default_path_to_charge_full() -> String {
	"/sys/class/power_supply/BAT0/charge_full".to_string()
}

fn default_path_to_status() -> String {
	"/sys/class/power_supply/BAT0/status".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
	Charging,
	Discharging,
	Full,
	NotCharging,
	Unknown,
}

impl TryFrom<&str> for Status {
	type Error = Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value.trim() {
			"Charging" => Ok(Status::Charging),
			"Discharging" => Ok(Status::Discharging),
			"Full" => Ok(Status::Full),
			"Not charging" => Ok(Status::NotCharging),
			"Unknown" => Ok(Status::Unknown),
			e => Err(Error::Parse {
				name: "Battery",
				reason: format!("Unknown battery status '{e}'"),
			}),
		}
	}
}

/// Given a percentage of charge, wrap the string `string` in an appropriate colour.
fn wrap_in_colour(string: &str, fraction: f32) -> String {
	let colour = if fraction > 0.5 {
		format!("{:0>2x}ff00", 255 - (510.0 * (fraction - 0.5)) as i32)
	} else {
		format!("ff{:0>2x}00", (510.0 * fraction) as i32)
	};
	format!("<span foreground='#{}'>{}</span>", colour, string)
}

/// Given a percentage of charge, return an appropriate battery symbol.
fn get_discharge_symbol(fraction: f32) -> &'static str {
	if fraction > 0.90 {
		"  "
	} else if fraction > 0.60 {
		"  "
	} else if fraction > 0.40 {
		"  "
	} else if fraction > 0.10 {
		"  "
	} else {
		"  "
	}
}

fn get_symbol(status: Status, fraction: f32) -> String {
	let string = match status {
		Status::Discharging => get_discharge_symbol(fraction),
		_ => " ",
	};
	wrap_in_colour(string, fraction)
}

/// Convert a float of minutes into a string of hours and minutes.
fn minutes_to_string(total: f32) -> String {
	let (mut hrs, mut mins) = (total / 60.0, total % 60.0);
	if mins >= 59.5 {
		hrs += 1.0;
		mins = 0.0;
	} else {
		mins = mins.round();
	}
	format!("{:.0}h{:02.0}m", hrs.floor(), mins)
}

struct Interval {
	then: Instant,
}

impl Interval {
	fn new() -> Self {
		Self {
			then: Instant::now(),
		}
	}

	fn elapsed(&mut self) -> f32 {
		let now = Instant::now();
		let elapsed = now.duration_since(self.then).as_secs() as f32 / 60.0;
		self.then = now;
		elapsed
	}
}

enum Remaining {
	Minutes(util::Ema<f32>),
	Calculating(f32),
}

impl Remaining {
	fn new(alpha: f32) -> Self {
		Self::Calculating(alpha)
	}

	fn push(&mut self, value: f32) -> f32 {
		match self {
			Self::Minutes(ema) => ema.push(value),
			Self::Calculating(alpha) => {
				let mut ema = util::Ema::new(*alpha);
				let value = ema.push(value);
				*self = Remaining::Minutes(ema);
				value
			}
		}
	}
}

impl fmt::Display for Remaining {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let string = match self {
			Self::Minutes(ema) => {
				if let Some(value) = Into::<Option<f32>>::into(ema) {
					&minutes_to_string(value)
				} else {
					"..."
				}
			}
			Self::Calculating(_) => "...",
		};
		write!(f, "{string}")
	}
}

impl IntoStream for Battery {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		stream! {
			let mut charge_watcher = Box::pin(util::watch(&self.path_to_charge_now, self.period));
			let mut status_watcher = Box::pin(util::watch(&self.path_to_status, self.period));
			let max: f32 = util::read_to_ty("Battery", self.path_to_charge_full).await.unwrap();
			let charge: f32 = charge_watcher.next().await
				.unwrap()
				.expect(&format!("couldn't read '{}'", self.path_to_charge_now))
				.trim()
				.parse()
				.unwrap();
			let mut charge_fraction = charge / max;
			let mut status: Status = status_watcher.next().await
				.unwrap()
				.expect(&format!("couldn't read '{}'", self.path_to_status))
				.as_str()
				.try_into()
				.unwrap();
			let mut prev_charge: Option<f32> = None;
			let mut interval = Interval::new();
			let mut remaining = Remaining::new(self.alpha);
			loop {
				tokio::select! {
					Some(new_charge) = charge_watcher.next() => {
						let elapsed = interval.elapsed();
						let new_charge = new_charge.unwrap().trim().parse().unwrap(); // TODO
						charge_fraction = new_charge / max;
						if let Some(prev_charge) = prev_charge.replace(new_charge) {
							let gap = match status {
								Status::Charging => max - new_charge,
								Status::Discharging => new_charge,
								Status::Full => 0.0,
								Status::NotCharging => new_charge,
								Status::Unknown => new_charge,
							};
							let rate = (prev_charge - new_charge).abs() / elapsed;
							remaining.push(gap / rate);
						} else {
							remaining = Remaining::new(self.alpha);
						};
					},
					Some(new_status) = status_watcher.next() => {
						status = new_status.unwrap().as_str().try_into().unwrap(); // TODO
						remaining = Remaining::new(self.alpha);
						interval = Interval::new();
						prev_charge = None;
					},
				}
				// TODO: We don't correctly report when the battery is full if the power is unplugged and
				// plugged in again when the battery is full.
				let symbol = get_symbol(status, charge_fraction);
				let percent = 100.0 * charge_fraction;
				yield Ok(format!("{symbol} {percent:.0}% ({remaining})"));
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::blocks::Block;
	use crate::config;

	#[test]
	fn configuration() {
		let string = "
			[Battery]
			period = 300
			alpha = 0.1
		";
		let mut deserialised = config::deserialise(string).unwrap();
		if let Block::Battery(battery) = deserialised.remove(0) {
			assert_eq!(battery.period, 300);
			assert_eq!(battery.alpha, 0.1);
		} else {
			panic!()
		};
	}

	#[test]
	fn minutes_to_string_works() {
		assert_eq!(minutes_to_string(302.2), "5h02m");
		assert_eq!(minutes_to_string(302.7), "5h03m");
		assert_eq!(minutes_to_string(60.0), "1h00m");
		assert_eq!(minutes_to_string(59.99), "1h00m");
		assert_eq!(minutes_to_string(60.5), "1h01m");
		assert_eq!(minutes_to_string(60.4999), "1h00m");
		assert_eq!(minutes_to_string(39.5), "0h40m");
	}

	#[test]
	fn test_wrap_in_colour() {
		let result = wrap_in_colour("a", 1.0);
		assert_eq!(result, "<span foreground=\'#00ff00\'>a</span>");

		let result = wrap_in_colour("a", 0.01);
		assert_eq!(result, "<span foreground=\'#ff0500\'>a</span>");
	}
}
