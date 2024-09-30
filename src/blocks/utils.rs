use async_stream::stream;
use futures_core::stream::Stream;
use std::io;
use std::path::Path;
use tokio::time::{sleep, Duration};

pub fn watch<P>(path: P, millis: u64) -> impl Stream<Item = io::Result<String>>
where
	P: AsRef<Path> + Copy,
{
	stream! {
		let mut current = "".to_string();
		loop {
			let new = tokio::fs::read_to_string(path).await?; // TODO: optimisation possible?
			if new != current {
				current = new;
				yield Ok(current.clone().into());
			}
			sleep(Duration::from_millis(millis)).await;
		}
	}
}
