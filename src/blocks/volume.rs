use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Volume {
	#[serde(default = "default_period")]
	period: u64,
	#[serde(default = "default_update_signal")]
	update_signal: i32,
}

fn default_period() -> u64 {
	2000
}

fn default_update_signal() -> i32 {
	signal_hook::consts::SIGUSR2
}
