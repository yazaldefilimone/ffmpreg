use crate::core::time::Time;

use super::format::PixelFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyframe {
	Key,
	NonKey,
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub pixel: PixelFormat,
	pub keyframe: Keyframe,
	pub pts: Option<Time>,
}
