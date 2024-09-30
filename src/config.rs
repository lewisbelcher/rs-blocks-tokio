use crate::blocks::Block;
use crate::error::Error;
use toml::Value;

fn map_block((name, value): (String, Value)) -> Result<Block, Error> {
	// In scope to have access to `value`
	macro_rules! map_block_arm {
		($name:ident) => {
			Block::$name(value.try_into().map_err(|e| Error::Deserialize {
				name: stringify!($name),
				reason: e.to_string(),
			})?)
		};
	}

	let block = match name.as_str() {
		"Battery" => map_block_arm!(Battery),
		"Brightness" => map_block_arm!(Brightness),
		"Cpu" => map_block_arm!(Cpu),
		"Memory" => map_block_arm!(Memory),
		"Network" => map_block_arm!(Network),
		"Time" => map_block_arm!(Time),
		"Volume" => map_block_arm!(Volume),
		_ => panic!("Oh no"),
	};
	Ok(block)
}

pub fn deserialise(string: &str) -> Result<Vec<Block>, Error> {
	// Why doesn't this work???
	// let deserialised: toml::map::Map<String, blocks::Block> = toml::from_str(string).unwrap();
	let deserialised: toml::map::Map<String, Value> = toml::from_str(string)?;
	deserialised.into_iter().map(map_block).collect()
}

#[cfg(test)]
mod test {
	use super::*;
	use std::matches;

	#[test]
	fn order_is_preserved() {
		let string = "
			[Volume]
			period = 10

			[Battery]
			period = 300
			alpha = 0.1
		";

		let deserialised = deserialise(string).unwrap();
		assert!(matches!(&deserialised[0], &Block::Volume(_)));
		assert!(matches!(&deserialised[1], &Block::Battery(_)));
	}
}
