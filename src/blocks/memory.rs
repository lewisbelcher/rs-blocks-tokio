use serde::Deserialize;

const PATTERN: &str = r"(?s)MemTotal:\s+(\d+).+MemFree:\s+(\d+)";

#[derive(Debug, Deserialize)]
pub struct Memory {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_alpha")]
	alpha: f32,
	#[serde(default = "default_meminfo_path")]
	meminfo_path: &'static str,
}

fn default_period() -> u64 {
	600
}

fn default_alpha() -> f32 {
	0.7
}

fn default_meminfo_path() -> &'static str {
	"/proc/meminfo"
}

struct MemStats {}
