use super::AdpcmState;
use crate::container::WavFormat;
use crate::core::{Decoder, Frame, FrameAudio, Packet};
use crate::io::IoResult;

pub struct AdpcmDecoder {
	format: WavFormat,
	states: Vec<AdpcmState>,
}

impl AdpcmDecoder {
	pub fn new(format: WavFormat) -> Self {
		let states = (0..format.channels).map(|_| AdpcmState::new()).collect();
		Self { format, states }
	}
}

impl Decoder for AdpcmDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		if packet.data.is_empty() {
			return Ok(None);
		}

		let channels = self.format.channels as usize;
		let mut output = Vec::with_capacity(packet.data.len() * 4);

		for (i, byte) in packet.data.iter().enumerate() {
			let channel = (i * 2) % channels;

			let low_nibble = byte & 0x0F;
			let sample1 = self.states[channel].decode_sample(low_nibble);
			output.extend_from_slice(&sample1.to_le_bytes());

			let high_nibble = (byte >> 4) & 0x0F;
			let channel2 = ((i * 2) + 1) % channels;
			let sample2 = self.states[channel2].decode_sample(high_nibble);
			output.extend_from_slice(&sample2.to_le_bytes());
		}

		let nb_samples = output.len() / 2 / channels;
		let audio = FrameAudio::new(output, self.format.sample_rate, self.format.channels)
			.with_nb_samples(nb_samples);
		let frame = Frame::new_audio(audio, packet.timebase, packet.stream_index).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
