use crate::container::Y4mFormat;
use crate::core::{Decoder, Frame, FrameVideo, Packet, VideoFormat};
use crate::io::IoResult;

pub struct RawVideoDecoder {
	format: Y4mFormat,
}

impl RawVideoDecoder {
	pub fn new(format: Y4mFormat) -> Self {
		Self { format }
	}
}

impl Decoder for RawVideoDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let video =
			FrameVideo::new(packet.data, self.format.width, self.format.height, VideoFormat::YUV420);
		let frame = Frame::new_video(video, packet.timebase, packet.stream_index).with_pts(packet.pts);
		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
