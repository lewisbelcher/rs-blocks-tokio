use crate::Error;
use async_stream::try_stream;
use futures_util::Stream;
use serde::Serialize;
use std::pin::Pin;

pub mod battery;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod stream_ext;
pub mod time;
pub mod util;
pub mod volume;

pub use battery::Battery;
pub use brightness::Brightness;
pub use cpu::Cpu;
pub use memory::Memory;
pub use network::Network;
pub use stream_ext::StreamExt2;
pub use time::Time;
pub use volume::Volume;

pub mod prelude {
	pub use super::{GetMarkup, GetName, IntoSerialized, IntoStream};
}

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

pub struct BlockResult {
	pub block_name: String,
	pub text: String,
}

pub trait IntoStream {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>>;

	fn into_stream_pin(self) -> Pin<Box<dyn Stream<Item = Result<BlockResult, Error>>>>
	where
		Self: 'static + GetName + IntoSerialized + Sized,
	{
		Box::pin(try_stream! {
			for await result in self.into_stream() {
				let block_name = Self::get_name().to_string();
				let full_text = match result {
					Ok(full_text) => full_text,
					Err(e) => e.to_string(),
				};
				yield BlockResult { block_name, text: Self::into_serialized(full_text)? };
			}
		})
	}
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
	pub fn into_stream_pin(self) -> Pin<Box<dyn Stream<Item = Result<BlockResult, Error>>>> {
		match self {
			Block::Battery(x) => x.into_stream_pin(),
			Block::Brightness(x) => x.into_stream_pin(),
			Block::Cpu(x) => x.into_stream_pin(),
			Block::Memory(x) => x.into_stream_pin(),
			Block::Network(x) => x.into_stream_pin(),
			Block::Time(x) => x.into_stream_pin(),
			Block::Volume(x) => x.into_stream_pin(),
		}
	}

	pub fn get_name(&self) -> &'static str {
		match self {
			Block::Battery(_) => Battery::get_name(),
			Block::Brightness(_) => Brightness::get_name(),
			Block::Cpu(_) => Cpu::get_name(),
			Block::Memory(_) => Memory::get_name(),
			Block::Network(_) => Network::get_name(),
			Block::Time(_) => Time::get_name(),
			Block::Volume(_) => Volume::get_name(),
		}
	}
}

pub fn default_alpha() -> f32 {
	0.1
}

pub fn default_period() -> u64 {
	700
}
