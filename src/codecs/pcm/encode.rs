use crate::core::{Encoder, Frame, Packet, Timebase};
use crate::io::IoResult;

pub struct PcmEncoder {
	timebase: Timebase,
}

impl PcmEncoder {
	pub fn new(timebase: Timebase) -> Self {
		Self { timebase }
	}
}

impl Encoder for PcmEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		match frame.data {
			crate::core::FrameData::Audio(audio) => {
				let packet = Packet::new(audio.data, frame.stream_index, self.timebase).with_pts(frame.pts);
				Ok(Some(packet))
			}
			crate::core::FrameData::Video(video) => {
				let packet = Packet::new(video.data, frame.stream_index, self.timebase).with_pts(frame.pts);
				Ok(Some(packet))
			}
		}
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
