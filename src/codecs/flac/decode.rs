use super::FlacStreamInfo;
use super::frame::decode_frame;
use crate::container::FlacFormat;
use crate::core::{Decoder, Frame, FrameAudio, Packet};
use crate::io::IoResult;

pub struct FlacDecoder {
	stream_info: FlacStreamInfo,
}

impl FlacDecoder {
	pub fn new(format: &FlacFormat) -> Self {
		let stream_info = FlacStreamInfo {
			min_block_size: format.min_block_size,
			max_block_size: format.max_block_size,
			min_frame_size: format.min_frame_size,
			max_frame_size: format.max_frame_size,
			sample_rate: format.sample_rate,
			channels: format.channels,
			bits_per_sample: format.bits_per_sample,
			total_samples: format.total_samples,
		};
		Self { stream_info }
	}

	pub fn from_stream_info(stream_info: FlacStreamInfo) -> Self {
		Self { stream_info }
	}

	fn samples_to_bytes(&self, samples: &[Vec<i32>]) -> Vec<u8> {
		let channels = samples.len();
		if channels == 0 {
			return Vec::new();
		}

		let block_size = samples[0].len();
		let bytes_per_sample = ((self.stream_info.bits_per_sample + 7) / 8) as usize;
		let mut output = Vec::with_capacity(block_size * channels * bytes_per_sample);

		for i in 0..block_size {
			for ch in 0..channels {
				let sample = samples[ch][i];
				match bytes_per_sample {
					1 => {
						output.push(sample as u8);
					}
					2 => {
						output.extend_from_slice(&(sample as i16).to_le_bytes());
					}
					3 => {
						output.push(sample as u8);
						output.push((sample >> 8) as u8);
						output.push((sample >> 16) as u8);
					}
					4 => {
						output.extend_from_slice(&sample.to_le_bytes());
					}
					_ => {}
				}
			}
		}

		output
	}
}

impl Decoder for FlacDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let flac_frame = decode_frame(&packet.data, &self.stream_info)?;

		let output = self.samples_to_bytes(&flac_frame.samples);
		let nb_samples = flac_frame.block_size;

		let audio = FrameAudio::new(output, self.stream_info.sample_rate, self.stream_info.channels)
			.with_nb_samples(nb_samples);

		let frame = Frame::new_audio(audio, packet.timebase, packet.stream_index).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
