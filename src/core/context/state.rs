use crate::core::{Channels, SampleFormat, SampleLayout, Time, VideoFrame, video};

#[derive(Debug, Clone)]
pub struct CodecState {
	pub extradata: Vec<u8>,
	pub pts: Option<Time>,
	pub dts: Option<Time>,
	pub last_pts: Option<Time>,
}

#[derive(Debug, Clone)]
pub struct VideoState {
	pub width: u32,
	pub height: u32,
	pub pixel: video::Pixel,
	pub reorder: Vec<VideoFrame>,
	pub has_keyframe: bool,

	pub codec_state: CodecState,
}

#[derive(Debug, Clone)]
pub struct AudioState {
	pub sample_rate: u32,
	pub channels: Channels,
	pub format: SampleFormat,
	pub layout: SampleLayout,
	pub frame_size: usize,
	pub codec_state: CodecState,

	pub samples_consumed: u64,
	pub samples_produced: u64,

	pub buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum State {
	Audio(AudioState),
	Video(VideoState),
}
