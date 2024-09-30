use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Time {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_format")]
	format: String,
}

fn default_period() -> u64 {
	1000
}

fn default_format() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}
