use crate::core::{Frame, Transform};
use crate::io::IoResult;

#[derive(Debug, Clone, Copy)]
pub enum ChannelLayout {
	Mono,
	Stereo,
}

pub struct ChannelMixer {
	target_layout: ChannelLayout,
}

impl ChannelMixer {
	pub fn new(target_layout: ChannelLayout) -> Self {
		Self { target_layout }
	}

	pub fn mono_to_stereo() -> Self {
		Self::new(ChannelLayout::Stereo)
	}

	pub fn stereo_to_mono() -> Self {
		Self::new(ChannelLayout::Mono)
	}

	fn convert_mono_to_stereo(samples: &[i16]) -> Vec<i16> {
		let mut output = Vec::with_capacity(samples.len() * 2);
		for &sample in samples {
			output.push(sample);
			output.push(sample);
		}
		output
	}

	fn convert_stereo_to_mono(samples: &[i16]) -> Vec<i16> {
		let mut output = Vec::with_capacity(samples.len() / 2);
		for pair in samples.chunks(2) {
			if pair.len() == 2 {
				let mixed = ((pair[0] as i32 + pair[1] as i32) / 2) as i16;
				output.push(mixed);
			}
		}
		output
	}
}

impl Transform for ChannelMixer {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let src_channels = audio_frame.channels;
			let target_channels = match self.target_layout {
				ChannelLayout::Mono => 1,
				ChannelLayout::Stereo => 2,
			};

			if src_channels == target_channels {
				return Ok(frame);
			}

			let input_samples: Vec<i16> =
				audio_frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

			let output_samples = match (src_channels, target_channels) {
				(1, 2) => Self::convert_mono_to_stereo(&input_samples),
				(2, 1) => Self::convert_stereo_to_mono(&input_samples),
				_ => input_samples,
			};

			let output_data: Vec<u8> = output_samples.iter().flat_map(|s| s.to_le_bytes()).collect();

			let nb_samples = output_samples.len() / target_channels as usize;

			audio_frame.data = output_data;
			audio_frame.channels = target_channels;
			audio_frame.nb_samples = nb_samples;
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"channel_mixer"
	}
}
