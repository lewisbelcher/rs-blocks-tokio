use std::io;

const USAGE: &str = "Rust Blocks 0.1.0
Lewis B. <gitlab.io/lewisbelcher>
A simple i3blocks replacement written in Rust.

USAGE:
    rs-blocks <CONFIG>

ARGS:
    <CONFIG>         Config file to use
";

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("failed to deserialise block '{name}': {reason}")]
	Deserialize { name: &'static str, reason: String },
	#[error("no block implemented for '{0}'")]
	InvalidBlockName(String),
	#[error(transparent)]
	Io(#[from] io::Error),
	#[error("error while parsing to type '{ty}': {reason}")]
	Parse { ty: &'static str, reason: String },
	#[error(transparent)]
	Serialize(#[from] serde_json::Error),
	#[error(transparent)]
	Toml(#[from] toml::de::Error),
	#[error("{}", USAGE)]
	Usage,
}
