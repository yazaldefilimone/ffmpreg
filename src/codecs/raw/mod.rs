pub mod decoder;
pub mod encoder;

use crate::core::*;

pub fn audio_state(params: &Parameters) -> State {
	let codec_state = CodecState { extradata: Vec::new(), pts: None, dts: None, last_pts: None };
	State::Audio(AudioState {
		sample_rate: params.sample_rate.unwrap_or(48_000),
		channels: Channels::from_count(params.channels.unwrap_or(2)),
		format: SampleFormat::S16,
		layout: SampleLayout::Interleaved,
		frame_size: 0,
		codec_state,
		samples_consumed: 0,
		samples_produced: 0,
		buffer: Vec::new(),
	})
}

pub fn video_state(params: &Parameters) -> State {
	let codec_state = CodecState { extradata: Vec::new(), pts: None, dts: None, last_pts: None };
	let pixel = Pixel { depth: 8, format: video::PixelFormat::YUV420 };
	State::Video(VideoState {
		width: params.width.unwrap_or(0),
		height: params.height.unwrap_or(0),
		pixel,
		reorder: Vec::new(),
		has_keyframe: false,
		codec_state,
	})
}

pub fn default_codec_id(kind: StreamKind) -> CodecId {
	match kind {
		StreamKind::Audio => CodecId::new("pcm_s16le"),
		StreamKind::Video => CodecId::new("yuv420p"),
		StreamKind::Subtitle => CodecId::new("text"),
		StreamKind::Other => CodecId::new("binary"),
	}
}

pub fn default_time_base(kind: StreamKind) -> TimeBase {
	match kind {
		StreamKind::Audio => TimeBase::new(1, 48_000),
		_ => TimeBase::new(1, 1_000),
	}
}

pub fn packet_time(index: u64, kind: StreamKind) -> Time {
	Time::new(index, default_time_base(kind))
}
