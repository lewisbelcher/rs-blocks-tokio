pub use blocks::{Block, IntoSerialized};
pub use error::Error;
use futures_util::pin_mut;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::fs;
use tokio::select;

pub mod args;
pub mod blocks;
pub mod config;
pub mod error;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
	let args = args::parse_args()?;
	let config_str = fs::read_to_string(args.config_path)?;
	let deserialised = config::deserialise(&config_str)?;

	let s1 = blocks::utils::watch("/proc/meminfo", 2300);
	let s2 = blocks::utils::watch("/sys/class/power_supply/BAT0/energy_now", 410);
	pin_mut!(s1, s2);
	let mut dict: HashMap<String, String> = HashMap::new();
	loop {
		select! {
			Some(Ok(val)) = s1.next() => dict.insert("s1".to_string(), "s1".to_string()),
			Some(Ok(val)) = s2.next() => dict.insert("s2".to_string(), "s2".to_string()),
		};
		println!("{}", serde_json::to_string(&dict).unwrap());
	}
	// Loop and select across all using `FuturesUnordered`
	// https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html
	Ok(())
}
