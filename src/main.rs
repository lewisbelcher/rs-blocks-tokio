pub use blocks::{Block, IntoSerialized};
pub use error::Error;
use std::fs;

pub mod args;
pub mod blocks;
pub mod config;
pub mod error;

fn main() -> Result<(), error::Error> {
	let args = args::parse_args()?;
	let config_str = fs::read_to_string(args.config_path)?;
	let deserialised = config::deserialise(&config_str)?;

	// TODO: Use <as> or something to solve this?
	let json = match deserialised[0] {
		Block::Battery(ref inner) => inner.into_json(),
		Block::Brightness(ref inner) => inner.into_json(),
		_ => todo!(),
	};
	println!("{}", json.unwrap());

	// Loop and select across all using `FuturesUnordered`
	// https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html
	Ok(())
}
