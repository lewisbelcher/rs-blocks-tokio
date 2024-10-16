use crate::Error;
use async_stream::stream;
use futures_util::Stream;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Sub};
use std::path::Path;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Ema<T: PartialEq> {
	current: Option<T>,
	alpha: T,
}

impl<T> Ema<T>
where
	T: From<u8> + Copy + Mul<Output = T> + Add<Output = T> + Sub<Output = T> + PartialEq,
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

impl<T: Copy + PartialEq> From<&Ema<T>> for Option<T> {
	fn from(ema: &Ema<T>) -> Self {
		ema.current
	}
}

// We want to display `current`. So we defer the Display trait to `T`
impl<T> Display for Ema<T>
where
	T: Display + Copy + PartialEq,
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
	// TODO: Implement opening a file and just reading X bytes to prevent the Memory block from
	// always updating
	stream! {
		let mut current = "".to_string();
		loop {
			let new = tokio::fs::read_to_string(path).await.map_err(Error::Io)?; // TODO: optimisation possible?
			if new != current {
				current = new;
				yield Ok(current.clone());
			}
			sleep(Duration::from_millis(millis)).await;
		}
	}
}

/// Read to Type
///
/// A convenience function to read a file to a given type `T`.
pub async fn read_to_ty<P, T>(name: &'static str, path: P) -> Result<T, Error>
where
	P: AsRef<Path> + Display,
	T: FromStr,
{
	let contents = tokio::fs::read_to_string(&path).await.map_err(Error::Io)?;
	contents.trim().parse().map_err(|_| Error::Parse {
		name,
		reason: format!(
			"couldn't convert contents of '{path}' to {}",
			std::any::type_name::<T>()
		),
	})
}

/// From String
///
/// A convenience function for converting a regex and string into a given type with
/// relevant error handling.
pub fn from_string<'a, T>(
	re: &regex::Regex,
	contents: &'a str,
	name: &'static str,
) -> Result<T, Error>
where
	T: TryFrom<regex::Captures<'a>, Error = String>,
{
	re.captures(contents)
		.ok_or_else(|| Error::Parse {
			name,
			reason: "regex pattern match failed".to_string(),
		})
		.map(|x| x.try_into())?
		.map_err(|e| Error::Parse { name, reason: e })
}
