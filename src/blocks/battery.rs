use crate::blocks::{default_period, prelude::*, util};
use crate::Error;
use async_stream::try_stream;
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
	path_to_charge_now: String,
	path_to_charge_full: String,
	path_to_status: String,
}

fn default_alpha() -> f32 {
	0.05
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
	Charging(Remaining),
	Discharging(Remaining),
	Full,
	NotCharging,
	Unknown,
}

impl Status {
	fn push(&mut self, max: f32, charge: f32, rate: f32) {
		match self {
			Status::Charging(ref mut rem) => rem.push((max - charge) / rate),
			Status::Discharging(ref mut rem) => rem.push(charge / rate),
			Status::Full => {}
			Status::NotCharging => {}
			Status::Unknown => {}
		};
	}
}

impl TryFrom<(&str, f32)> for Status {
	type Error = Error;

	fn try_from((value, alpha): (&str, f32)) -> Result<Self, Self::Error> {
		let status = match value.trim() {
			"Charging" => Status::Charging(Remaining::new(alpha)),
			"Discharging" => Status::Discharging(Remaining::new(alpha)),
			"Full" => Status::Full,
			"Not charging" => Status::NotCharging,
			"Unknown" => Status::Unknown,
			e => {
				return Err(Error::Parse {
					name: "Battery",
					reason: format!("Unknown battery status '{e}'"),
				})
			}
		};
		Ok(status)
	}
}

impl fmt::Display for Status {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let string = match self {
			Self::Charging(rem) | Self::Discharging(rem) => &format!("{rem}"),
			Self::Full => "Full",
			Self::NotCharging => "NotCharging",
			Self::Unknown => "Unknown",
		};
		write!(f, "{string}")
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
		Status::Discharging(_) => get_discharge_symbol(fraction),
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

#[derive(PartialEq, Debug, Clone, Copy)]
enum Remaining {
	Minutes(util::Ema<f32>),
	Calculating(f32),
}

impl Remaining {
	fn new(alpha: f32) -> Self {
		Self::Calculating(alpha)
	}

	fn push(&mut self, value: f32) {
		match self {
			Self::Minutes(ema) => {
				ema.push(value);
			}
			Self::Calculating(alpha) => {
				let mut ema = util::Ema::new(*alpha);
				ema.push(value);
				*self = Remaining::Minutes(ema);
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
		// We want to run these on the current thread and blocking during stream setup. Apparently
		// tokio's `Handle::block_on` is error-prone when using `current_thread`. See
		// https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on
		// But `futures::executor::block_on` works and executes on the current thread.
		let max: f32 = {
			let future = util::read_to_ty("Battery", &self.path_to_charge_full);
			futures::executor::block_on(future).unwrap()
		};
		let mut charge_fraction: f32 = {
			let future = util::read_to_ty("Battery", &self.path_to_charge_now);
			let charge: f32 = futures::executor::block_on(future).unwrap();
			charge / max
		};
		let mut status: Status = {
			let future = util::read_to_ty("Battery", &self.path_to_status);
			let status_str: String = futures::executor::block_on(future).unwrap();
			(status_str.as_str(), self.alpha).try_into().unwrap()
		};

		try_stream! {
			let mut charge_watcher = Box::pin(util::watch(&self.path_to_charge_now, self.period));
			let mut status_watcher = Box::pin(util::watch(&self.path_to_status, self.period));
			let mut prev_charge: Option<f32> = None;
			let mut interval = Interval::new();
			loop {
				tokio::select! {
					Some(new_charge) = charge_watcher.next() => {
						let elapsed = interval.elapsed();
						// We'd rather do this: `let new_charge = new_charge?.trim().parse().map_err(...)?;`
						// But there appears to be an issue with using `select` nested in `stream`. See
						// https://github.com/tokio-rs/async-stream/issues/63
						// We could also drop the extra `if let` below and returning the `new_charge.map(...)`
						// in that case
						let new_charge = new_charge
							.and_then(|x| x.trim().parse::<f32>()
								.map_err(|e| Error::Parse { name: Self::get_name(), reason: format!("{} '{x:?}'", e.to_string()) })
							);
						if let Ok(new_charge) = new_charge {
							charge_fraction = new_charge / max;
							if let Some(prev_charge) = prev_charge.replace(new_charge) {
								let rate = (prev_charge - new_charge).abs() / elapsed;
								status.push(max, new_charge, rate);
							}
						}
						new_charge.map(|_| ())
					},
					Some(new_status) = status_watcher.next() => {
						// Again, we'd rather do this: status = (new_status?.as_str(), self.alpha).try_into()?;
						// But we have the same issues as stated above...
						let new_status = new_status.and_then(|x| (x.as_str(), self.alpha).try_into());
						if let Ok(new_status) = new_status {
							status = new_status;
							interval = Interval::new();
							prev_charge = None;
						}
						new_status.map(|_| ())
					},
				}?;
				let symbol = get_symbol(status, charge_fraction);
				let percent = 100.0 * charge_fraction;
				yield format!("{symbol} {percent:.0}% ({status})");
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

	#[test]
	fn changing_remaining() {
		let mut remaining = Remaining::new(0.5);
		assert_eq!(remaining, Remaining::Calculating(0.5));
		remaining.push(1.0);
		let mut expected_ema = util::Ema::new(0.5);
		expected_ema.push(1.0);
		assert_eq!(remaining, Remaining::Minutes(expected_ema));
	}

	#[test]
	fn changing_status() {
		let mut status: Status = ("Discharging", 0.5).try_into().unwrap();
		match status {
			Status::Discharging(rem) => assert_eq!(rem, Remaining::Calculating(0.5)),
			_ => panic!("should not get here"),
		}
		status.push(1.0, 1.0, 1.0);
		let mut expected_ema = util::Ema::new(0.5);
		expected_ema.push(1.0);
		match status {
			Status::Discharging(rem) => assert_eq!(rem, Remaining::Minutes(expected_ema)),
			_ => panic!("should not get here"),
		}
	}
}
