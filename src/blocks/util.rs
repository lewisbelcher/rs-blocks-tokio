use crate::Error;
use async_stream::stream;
use futures_util::Stream;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Sub};
use std::path::Path;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;

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
		current = self.alpha * new + (T::from(1) - self.alpha) * current;
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

pub fn watch<P, const CAPACITY: usize>(path: P) -> impl Stream<Item = Result<String, Error>>
where
	P: AsRef<Path> + Copy,
{
	stream! {
		let mut file = File::open(path).await?;
		let mut buf = [0; CAPACITY];
		let mut current = [0; CAPACITY];
		loop {
			let n = file.read(&mut buf[..]).await?;
			if buf != current {
				current = buf;
				yield std::str::from_utf8(&current[..n])
					.map(|x| x.to_owned())
					.map_err(|e| Error::Parse { ty: "UTF-8 string", reason: e.to_string() });
			}
			file.seek(SeekFrom::Start(0)).await?;
		}
	}
}

/// Read to Type
///
/// A convenience function to read a file to a given type `T`.
pub async fn read_to_ty<P, T>(path: P) -> Result<T, Error>
where
	P: AsRef<Path> + Display,
	T: FromStr,
	<T as FromStr>::Err: ToString,
{
	let contents = tokio::fs::read_to_string(&path).await?;
	contents
		.trim()
		.parse()
		.map_err(|e: <T as FromStr>::Err| Error::Parse {
			ty: std::any::type_name::<T>(),
			reason: e.to_string(),
		})
}

/// From String
///
/// A convenience function for converting a regex and string into a given type with
/// relevant error handling.
pub fn from_string<'a, T>(re: &regex::Regex, contents: &'a str) -> Result<T, Error>
where
	T: TryFrom<regex::Captures<'a>, Error = String>,
{
	re.captures(contents)
		.ok_or_else(|| Error::Parse {
			ty: std::any::type_name::<T>(),
			reason: "regex pattern match failed".to_string(),
		})
		.map(|x| x.try_into())?
		.map_err(|e| Error::Parse {
			ty: std::any::type_name::<T>(),
			reason: e,
		})
}
