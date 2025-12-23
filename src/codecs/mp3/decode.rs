use super::header::FrameHeader;
use super::layer3::Layer3Decoder;
use crate::core::{Decoder, Frame, FrameAudio, Packet};
use crate::io::IoResult;

pub struct Mp3Decoder {
	sample_rate: u32,
	channels: u8,
	layer3: Layer3Decoder,
	residual_data: Vec<u8>,
}

impl Mp3Decoder {
	pub fn new(sample_rate: u32, channels: u8) -> Self {
		Self {
			sample_rate,
			channels,
			layer3: Layer3Decoder::new(),
			residual_data: Vec::with_capacity(4096),
		}
	}

	pub fn from_header(data: &[u8]) -> Option<Self> {
		if data.len() < 4 {
			return None;
		}

		let header = FrameHeader::parse(data)?;
		Some(Self::new(header.sample_rate, header.channels))
	}
}

impl Decoder for Mp3Decoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		if packet.data.is_empty() {
			return Ok(None);
		}

		self.residual_data.extend_from_slice(&packet.data);

		let mut all_samples: Vec<i16> = Vec::new();
		let mut detected_sample_rate = self.sample_rate;
		let mut detected_channels = self.channels;
		let mut processed_bytes = 0;

		let mut offset = 0;
		loop {
			if offset + 4 > self.residual_data.len() {
				break;
			}

			// Try to parse frame header
			let header_data = &self.residual_data[offset..];
			match FrameHeader::parse(header_data) {
				Some(header) => {
					// Check if we have complete frame
					if offset + header.frame_size > self.residual_data.len() {
						break;
					}

					let frame_data = &self.residual_data[offset..offset + header.frame_size];

					// Decode frame
					if let Some(samples) = self.layer3.decode_frame(&header, frame_data) {
						all_samples.extend_from_slice(&samples);
						detected_sample_rate = header.sample_rate;
						detected_channels = header.channels;
						processed_bytes = offset + header.frame_size;
						offset += header.frame_size;
					} else {
						break;
					}
				}
				None => break,
			}
		}

		// Remove processed bytes from buffer
		if processed_bytes > 0 {
			self.residual_data.drain(0..processed_bytes);
		}

		if all_samples.is_empty() {
			return Ok(None);
		}

		let mut output = Vec::with_capacity(all_samples.len() * 2);
		for sample in &all_samples {
			output.extend_from_slice(&sample.to_le_bytes());
		}

		let nb_samples = all_samples.len() / detected_channels.max(1) as usize;
		let audio =
			FrameAudio::new(output, detected_sample_rate, detected_channels).with_nb_samples(nb_samples);
		let frame = Frame::new_audio(audio, packet.timebase, packet.stream_index).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
