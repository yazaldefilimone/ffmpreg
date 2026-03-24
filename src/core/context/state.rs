use crate::core::{Time, VideoFrame, audio::ChannelLayout, video::PixelFormat};

#[derive(Debug, Clone)]
pub struct CodecState {
	pub extradata: Vec<u8>,
	pub pts: Option<Time>,
	pub dts: Option<Time>,
	pub last_pts: Option<Time>,
}

#[derive(Debug, Clone)]
pub enum Samples {
	S16(Vec<i16>),
	S32(Vec<i32>),
	F32(Vec<f32>),
	F64(Vec<f64>),
}

#[derive(Debug, Clone)]
pub struct VideoState {
	pub width: u32,
	pub height: u32,
	pub pixel: PixelFormat,
	pub reorder: Vec<VideoFrame>,
	pub has_keyframe: bool,

	pub base: CodecState,
}

#[derive(Debug, Clone)]
pub struct AudioState {
	pub sample_rate: u32,
	pub channel: ChannelLayout,
	pub frame_size: usize,
	pub base: CodecState,
	pub samples: Samples,

	pub samples_consumed: u64,
	pub samples_produced: u64,

	pub buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum State {
	Audio(AudioState),
	Video(VideoState),
}
