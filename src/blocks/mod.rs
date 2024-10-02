use crate::Error;
use async_stream::stream;
use futures_util::pin_mut;
use futures_util::Stream;
use futures_util::StreamExt;
use serde::Serialize;

pub mod battery;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod time;
pub mod util;
pub mod volume;

pub use memory::Memory;

// TODO: Derive macro or attribute macro for `period` and `alpha`

pub trait GetName {
	fn get_name() -> &'static str;
}

pub trait GetMarkup {
	fn get_markup() -> Option<&'static str> {
		None
	}
}

/// Struct that will be serialised to produce a block. Note that there are many other attributes
/// we could introduce here, but these are the only ones being used at the moment.
#[derive(Serialize)]
pub struct Serialized {
	pub name: &'static str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub full_text: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub markup: Option<&'static str>,
}

pub trait IntoSerialized: GetName + GetMarkup {
	fn into_serialized(full_text: impl Into<Option<String>>) -> Result<String, Error> {
		let serialized = Serialized {
			name: Self::get_name(),
			full_text: full_text.into(),
			markup: Self::get_markup(),
		};
		serde_json::to_string(&serialized).map_err(Error::Serialize)
	}
}

pub trait IntoStream {
	// Should this consume self? If anything, just for semantic reasons?
	fn into_stream(&self) -> impl Stream<Item = Result<String, Error>>;
}

#[derive(Debug)]
pub enum Block {
	Battery(battery::Battery),
	Brightness(brightness::Brightness),
	Cpu(cpu::Cpu),
	Memory(memory::Memory),
	Network(network::Network),
	Time(time::Time),
	Volume(volume::Volume),
}

impl Block {
	pub fn into_stream(block: Block) -> impl Stream<Item = Result<(String, String), Error>> {
		match block {
			// Block::Battery(x) => x.into_stream(),
			// Block::Brightness(x) => x.into_stream(),
			// Block::Cpu(x) => x.into_stream(),
			Block::Memory(x) => {
				stream! {
					let block_stream = x.into_stream();
					pin_mut!(block_stream);
					while let Some(text) = block_stream.next().await {
						yield Ok((Memory::get_name().to_string(), Memory::into_serialized(text?)?));
					}
				}
			}
			// Block::Network(x) => x.into_stream(),
			// Block::Time(x) => x.into_stream(),
			// Block::Volume(x) => x.into_stream(),
			_ => unimplemented!(),
		}
	}
}
