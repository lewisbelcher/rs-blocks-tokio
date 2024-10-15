use futures_util::{stream::SelectAll, StreamExt};
use indexmap::IndexMap;
use std::fs;

pub mod args;
pub mod blocks;
pub mod config;
pub mod error;

pub use error::Error;

fn initialise_output_map(block_vec: &[blocks::Block]) -> IndexMap<String, String> {
	block_vec
		.iter()
		.map(|block| (block.get_name().to_string(), "{}".to_string()))
		.collect()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::Error> {
	let args = args::parse_args()?;
	let config_str = fs::read_to_string(args.config_path)?;
	let block_vec = config::deserialise(&config_str)?;
	let mut output_map = initialise_output_map(&block_vec);

	// TODO: Why doesn't FuturesUnordered work?
	// let futures: FuturesUnordered<_> = block_vec
	let mut futures: SelectAll<_> = block_vec
		.into_iter()
		.map(|block| block.into_stream())
		.collect();
	loop {
		let res = futures.select_next_some().await?;
		output_map.insert(res.0, res.1);
		println!("{}", serde_json::to_string(&output_map).unwrap());
	}
}
