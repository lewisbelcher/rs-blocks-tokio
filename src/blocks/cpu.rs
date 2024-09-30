use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Cpu {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.7
}
