use futures_util::{stream::SelectAll, StreamExt};
use indexmap::IndexMap;
use itertools::Itertools;
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

fn print_preamble() {
	println!("{{\"version\":1,\"click_events\":true}}");
	println!("[");
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
		.map(|block| block.into_stream_pin())
		.collect();
	print_preamble();
	loop {
		let res = futures.select_next_some().await?;
		output_map.insert(res.block_name, res.text);
		let print: String = output_map
			.values()
			.filter_map(|x| if x != "{}" { Some(x.as_str()) } else { None })
			.intersperse(",")
			.collect();
		println!("[{}],", print);
	}
}
