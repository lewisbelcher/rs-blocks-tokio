use futures_util::{stream::SelectAll, StreamExt};
use std::fs;

pub mod args;
pub mod blocks;
pub mod config;
pub mod error;

pub use error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::Error> {
	let args = args::parse_args()?;
	let config_str = fs::read_to_string(args.config_path)?;
	let deserialised = config::deserialise(&config_str)?;
	let mut dict: indexmap::IndexMap<String, String> = deserialised
		.iter()
		.map(|block| (block.get_name().to_string(), "{}".to_string()))
		.collect();

	// Target:
	// TODO: Why doesn't FuturesUnordered work?
	// let futures: FuturesUnordered<_> = deserialised
	let mut futures: SelectAll<_> = deserialised
		.into_iter()
		.map(|block| block.into_stream())
		.collect();
	loop {
		let res = futures.select_next_some().await?;
		dict.insert(res.0, res.1);
		println!("{}", serde_json::to_string(&dict).unwrap());
	}
}
