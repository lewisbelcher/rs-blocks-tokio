use crate::Error;
use futures_util::Stream;
use serde::Serialize;
use std::future::Future;

pub mod battery;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod time;
pub mod utils;
pub mod volume;

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
	fn into_serialized(&self, full_text: impl Into<Option<String>>) -> Result<String, Error> {
		let serialized = Serialized {
			name: Self::get_name(),
			full_text: full_text.into(),
			markup: Self::get_markup(),
		};
		serde_json::to_string(&serialized).map_err(Error::Serialize)
	}
}

pub trait IntoStream {
	fn into_stream(&mut self) -> impl Stream<Item = Result<String, Error>>;
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
