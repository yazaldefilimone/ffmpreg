use crate::core::{Encoder, Frame, Packet, Timebase};
use crate::io::IoResult;

pub struct RawVideoEncoder {
	timebase: Timebase,
}

impl RawVideoEncoder {
	pub fn new(timebase: Timebase) -> Self {
		Self { timebase }
	}
}

impl Encoder for RawVideoEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let data = match frame.data {
			crate::core::FrameData::Audio(audio) => audio.data,
			crate::core::FrameData::Video(video) => video.data,
		};
		let packet = Packet::new(data, frame.stream_index, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
