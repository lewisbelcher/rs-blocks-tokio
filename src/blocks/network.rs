use super::{default_alpha, default_period, prelude::*};
use crate::Error;
use async_stream::stream;
use futures_util::Stream;
use rs_blocks_macros::*;
use serde::Deserialize;
use std::path::Path;
use tokio::time::{self, Duration};

#[with_fields(alpha, period)]
#[derive(Debug, Deserialize, GetName, PangoMarkup, IntoSerialized)]
pub struct Network {
	path_to_rx: String,
	path_to_tx: String,
}

struct NetworkSpeed {
	curr: f32,
	prev: f32,
	coef: f32,
}

impl NetworkSpeed {
	fn new(coef: f32) -> NetworkSpeed {
		NetworkSpeed {
			curr: 0.0,
			prev: 0.0,
			coef,
		}
	}

	fn push(&mut self, new: f32) {
		self.prev = self.curr;
		self.curr = new;
	}

	fn calc_speed(&self) -> f32 {
		(self.curr - self.prev) * self.coef
	}
}

async fn read_file<P: AsRef<Path>>(path: P) -> Result<f32, Error> {
	let contents = tokio::fs::read_to_string(path).await.map_err(Error::Io)?;
	contents.trim().parse().map_err(|_| Error::Parse {
		name: "Network",
		ty: "f32",
	})
}

impl IntoStream for Network {
	fn into_stream(self) -> impl Stream<Item = Result<String, Error>> {
		let coef = 1.0 / (self.period as f32 * 1.024); // Report in kB/s (NB period is in ms)
		let mut rx = NetworkSpeed::new(coef);
		let mut tx = NetworkSpeed::new(coef);
		stream! {
			let mut interval = time::interval(Duration::from_millis(self.period));
			rx.push(read_file(&self.path_to_rx).await?);
			tx.push(read_file(&self.path_to_tx).await?);
			loop {
				interval.tick().await;
				rx.push(read_file(&self.path_to_rx).await?);
				tx.push(read_file(&self.path_to_tx).await?);
				yield Ok(format!(
					"<span foreground='#ccffcc'>  {:.1}</span> <span foreground='#ffcccc'>  {:.1}</span>",
					rx.calc_speed(),
					tx.calc_speed()
				));
			}
		}
	}
}
