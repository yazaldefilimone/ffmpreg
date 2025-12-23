use super::FlacStreamInfo;
use super::frame::encode_frame;
use crate::core::{Encoder, Frame, Packet, Timebase};
use crate::io::IoResult;

pub struct FlacEncoder {
	stream_info: FlacStreamInfo,
	timebase: Timebase,
	frame_count: u64,
}

impl FlacEncoder {
	pub fn new(sample_rate: u32, channels: u8, bits_per_sample: u8, block_size: u16) -> Self {
		let stream_info = FlacStreamInfo {
			min_block_size: block_size,
			max_block_size: block_size,
			min_frame_size: 0,
			max_frame_size: 0,
			sample_rate,
			channels,
			bits_per_sample,
			total_samples: 0,
		};
		let timebase = Timebase::new(1, sample_rate);
		Self { stream_info, timebase, frame_count: 0 }
	}

	pub fn from_stream_info(stream_info: FlacStreamInfo) -> Self {
		let timebase = Timebase::new(1, stream_info.sample_rate);
		Self { stream_info, timebase, frame_count: 0 }
	}

	fn bytes_to_samples(&self, data: &[u8]) -> Vec<Vec<i32>> {
		let channels = self.stream_info.channels as usize;
		let bytes_per_sample = ((self.stream_info.bits_per_sample + 7) / 8) as usize;
		let frame_size = channels * bytes_per_sample;

		if data.len() < frame_size || channels == 0 {
			return vec![Vec::new(); channels];
		}

		let num_samples = data.len() / frame_size;
		let mut channel_samples: Vec<Vec<i32>> =
			(0..channels).map(|_| Vec::with_capacity(num_samples)).collect();

		for i in 0..num_samples {
			for ch in 0..channels {
				let offset = i * frame_size + ch * bytes_per_sample;
				let sample = match bytes_per_sample {
					1 => data[offset] as i8 as i32,
					2 => i16::from_le_bytes([data[offset], data[offset + 1]]) as i32,
					3 => {
						let low = data[offset] as i32;
						let mid = data[offset + 1] as i32;
						let high = data[offset + 2] as i8 as i32;
						low | (mid << 8) | (high << 16)
					}
					4 => {
						i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
					}
					_ => 0,
				};
				channel_samples[ch].push(sample);
			}
		}

		channel_samples
	}
}

impl Encoder for FlacEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let data_bytes = match &frame.data {
			crate::core::FrameData::Audio(audio) => &audio.data,
			crate::core::FrameData::Video(video) => &video.data,
		};

		let samples = self.bytes_to_samples(data_bytes);

		if samples.is_empty() || samples[0].is_empty() {
			return Ok(None);
		}

		let encoded = encode_frame(&samples, self.frame_count, &self.stream_info);
		self.frame_count += 1;

		let packet = Packet::new(encoded, frame.stream_index, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
