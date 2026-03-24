use crate::core::time::Time;

use super::format::{ChannelLayout, SampleFormat, SampleRate};

#[derive(Debug, Clone)]
pub struct AudioFrame {
	pub data: Vec<u8>,
	pub sample_rate: SampleRate,
	pub channels: ChannelLayout,
	pub bit_depth: SampleFormat,
	pub nb_samples: usize,
	pub pts: Option<Time>,
}

impl AudioFrame {
	pub fn new(
		data: Vec<u8>,
		sample_rate: SampleRate,
		channels: ChannelLayout,
		bit_depth: SampleFormat,
	) -> Self {
		let nb_samples = data.len() / (channels.count() as usize * bit_depth.bytes());
		Self { data, sample_rate, channels, bit_depth, nb_samples, pts: None }
	}

	pub fn frame_size(&self) -> usize {
		self.nb_samples * self.channels.count() as usize * self.bit_depth.bytes()
	}
}
