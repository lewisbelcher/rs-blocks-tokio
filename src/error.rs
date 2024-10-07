use std::io;

const USAGE: &str = "Rust Blocks 1.0.0
Lewis B. <gitlab.io/lewisbelcher>
A simple i3blocks replacement written in Rust.

USAGE:
    rs-blocks <CONFIG>

ARGS:
    <CONFIG>         Config file to use
";

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("failed to deserialise {name} block: {reason}")]
	Deserialize { name: &'static str, reason: String },
	#[error("no block implemented for '{0}'")]
	InvalidBlockName(String),
	#[error(transparent)]
	Io(#[from] io::Error),
	#[error("failed to parse 'name' as '{ty}'")]
	Parse { name: &'static str, ty: &'static str },
	#[error("could not pattern match for block '{name}'")]
	PatternMatch { name: &'static str },
	#[error(transparent)]
	Serialize(#[from] serde_json::Error),
	#[error(transparent)]
	Toml(#[from] toml::de::Error),
	#[error("trying to access uninitialised block '{0}'")]
	UninitialisedBlock(String),
	#[error("{}", USAGE)]
	Usage,
}
