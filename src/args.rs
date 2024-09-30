use crate::error::Error;
use std::env;

pub struct Args {
	pub config_path: String,
}

pub fn parse_args() -> Result<Args, Error> {
	let mut args: Vec<String> = env::args().collect();
	match args.len() {
		2 => Ok(Args { config_path: args.remove(1) }),
		_ => Err(Error::Usage),
	}
}
