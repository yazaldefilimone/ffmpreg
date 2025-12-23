use super::AdpcmState;
use crate::core::{Encoder, Frame, Packet, Timebase};
use crate::io::IoResult;

pub struct AdpcmEncoder {
	timebase: Timebase,
	channels: usize,
	states: Vec<AdpcmState>,
}

impl AdpcmEncoder {
	pub fn new(timebase: Timebase, channels: u8) -> Self {
		let states = (0..channels).map(|_| AdpcmState::new()).collect();
		Self { timebase, channels: channels as usize, states }
	}
}

impl Encoder for AdpcmEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let data_bytes = match &frame.data {
			crate::core::FrameData::Audio(audio) => &audio.data,
			crate::core::FrameData::Video(video) => &video.data,
		};

		let samples: Vec<i16> =
			data_bytes.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

		let mut output = Vec::with_capacity(samples.len() / 2);

		for pair in samples.chunks(2) {
			let channel1 = 0 % self.channels;
			let nibble1 = self.states[channel1].encode_sample(pair[0]);

			let nibble2 = if pair.len() > 1 {
				let channel2 = 1 % self.channels;
				self.states[channel2].encode_sample(pair[1])
			} else {
				0
			};

			output.push(nibble1 | (nibble2 << 4));
		}

		let packet = Packet::new(output, frame.stream_index, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
