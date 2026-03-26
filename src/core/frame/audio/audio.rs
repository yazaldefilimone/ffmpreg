use crate::core::time::Time;

use super::format::{Channels, SampleFormat, SampleLayout};

#[derive(Debug, Clone)]
pub struct AudioFrame {
	pub data: Vec<u8>,
	pub sample_rate: u32,
	pub channels: Channels,
	pub format: SampleFormat,
	pub layout: SampleLayout,
	pub nb_samples: usize,
	pub pts: Option<Time>,
}

impl AudioFrame {
	pub fn new(
		data: Vec<u8>,
		sample_rate: u32,
		channels: Channels,
		format: SampleFormat,
		layout: SampleLayout,
	) -> Self {
		let nb_samples = match layout {
			SampleLayout::Interleaved => data.len() / (channels.count() as usize * format.bytes()),
			SampleLayout::Planar => data.len() / format.bytes(),
		};
		Self { data, sample_rate, channels, format, layout, nb_samples, pts: None }
	}

	pub fn frame_size(&self) -> usize {
		match self.layout {
			SampleLayout::Interleaved => {
				self.nb_samples * self.channels.count() as usize * self.format.bytes()
			}
			SampleLayout::Planar => self.nb_samples * self.format.bytes(),
		}
	}

	pub fn is_planar(&self) -> bool {
		self.layout.is_planar()
	}
}
