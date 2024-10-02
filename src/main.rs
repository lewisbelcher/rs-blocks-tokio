pub use error::Error;

use blocks::{Block, GetName, IntoSerialized, IntoStream, Memory};
use futures_util::stream::{FuturesUnordered, SelectAll};
use futures_util::{pin_mut, StreamExt};
use std::collections::HashMap;
use std::fs;
use tokio::select;

pub mod args;
pub mod blocks;
pub mod config;
pub mod error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::Error> {
	let args = args::parse_args()?;
	let config_str = fs::read_to_string(args.config_path)?;
	let mut deserialised = config::deserialise(&config_str)?;
	let mut dict: HashMap<String, String> = HashMap::new();

	// Target:
	// TODO: Why doesn't FuturesUnordered work?
	// let futures: FuturesUnordered<_> = deserialised
	let mut futures: SelectAll<_> = deserialised
		.into_iter()
		.map(|x| Box::pin(Block::into_stream(x)))
		.collect();
	loop {
		let res = futures.select_next_some().await?;
		dict.insert(res.0, res.1);
		println!("{}", serde_json::to_string(&dict).unwrap());
	}
}
