use crate::container::yuv::YuvFormat;
use crate::core::frame::video::PixelFormat;
use crate::core::frame::{BitDepth, Frame};
use crate::core::packet::{Packet, PacketIter};
use crate::core::time::Time;
use crate::core::track::{ColorSpace, Pixel, TrackFormat, VideoFormat};
use crate::core::traits::Encoder;
use crate::message::Result;

pub struct YuvEncoder {
	width: u32,
	height: u32,
	pixel: PixelFormat,
	fps: f64,
}

impl YuvEncoder {
	pub fn new(width: u32, height: u32, pixel: PixelFormat, fps: f64) -> Self {
		Self { width, height, pixel, fps }
	}

	pub fn from_format(format: &YuvFormat) -> Self {
		Self::new(format.width, format.height, format.pixel, format.fps)
	}
}

impl Encoder for YuvEncoder {
	fn format(&self) -> TrackFormat {
		TrackFormat::Video(VideoFormat {
			width: self.width,
			height: self.height,
			pixel: Pixel {
				depth: BitDepth::from_bits_any(self.pixel.depth),
				color_space: ColorSpace::YUV,
			},
		})
	}

	fn encode(&mut self, frame: Frame) -> Result<PacketIter> {
		let video = frame.video().ok_or_else(|| crate::error!("no video data"))?;
		let data = video.to_interleaved();

		let fps_num = (self.fps * 1000.0) as u32;
		let fps_den = 1000u32;
		let time = Time::new(fps_den, fps_num)?;

		let is_key = video.is_key();
		let packet =
			Packet::new(data, frame.track_id, time).with_pts(frame.pts().pts).with_keyframe(is_key);

		Ok(PacketIter::new(vec![packet]))
	}

	fn finish(&mut self) -> Result<PacketIter> {
		Ok(PacketIter::empty())
	}
}
