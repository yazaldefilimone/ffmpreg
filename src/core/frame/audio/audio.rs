use crate::core::time::Time;

use super::format::{Channels, SampleFormat, SampleRate};

#[derive(Debug, Clone)]
pub struct AudioFrame {
	pub data: Vec<u8>,
	pub sample_rate: SampleRate,
	pub channels: Channels,
	pub format: SampleFormat,
	pub nb_samples: usize,
	pub pts: Option<Time>,
}

impl AudioFrame {
	pub fn new(
		data: Vec<u8>,
		sample_rate: SampleRate,
		channels: Channels,
		format: SampleFormat,
	) -> Self {
		let nb_samples = data.len() / (channels.count() as usize * format.bytes());
		Self { data, sample_rate, channels, format, nb_samples, pts: None }
	}

	pub fn frame_size(&self) -> usize {
		self.nb_samples * self.channels.count() as usize * self.format.bytes()
	}
}
