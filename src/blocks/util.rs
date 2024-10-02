use crate::Error;
use async_stream::stream;
use futures_util::Stream;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Sub};
use std::path::Path;
use tokio::time::{sleep, Duration};

pub struct Ema<T> {
	current: Option<T>,
	alpha: T,
}

impl<T> Ema<T>
where
	T: From<u8> + Copy + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
{
	pub fn new(alpha: T) -> Ema<T> {
		Ema {
			current: None,
			alpha,
		}
	}

	pub fn push(&mut self, new: T) -> T {
		let mut current = self.current.unwrap_or(new);
		current = self.alpha * current + (T::from(1) - self.alpha) * new;
		self.current = Some(current);
		current
	}
}

// We want to display `current`. So we defer the Display trait to `T`
impl<T> Display for Ema<T>
where
	T: Display + Copy,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		if let Some(current) = self.current {
			current.fmt(f)
		} else {
			Err(fmt::Error)
		}
	}
}

pub fn watch<P>(path: P, millis: u64) -> impl Stream<Item = Result<String, Error>>
where
	P: AsRef<Path> + Copy,
{
	stream! {
		let mut current = "".to_string();
		loop {
			let new = tokio::fs::read_to_string(path).await.map_err(Error::Io)?; // TODO: optimisation possible?
			if new != current {
				current = new;
				yield Ok(current.clone().into());
			}
			sleep(Duration::from_millis(millis)).await;
		}
	}
}
