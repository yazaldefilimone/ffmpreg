pub mod decoder;
pub mod encoder;

use crate::core::*;

pub fn audio_state(parameters: &Parameters) -> State {
	State::Audio(AudioState {
		sample_rate: parameters.sample_rate.unwrap_or(48_000),
		channels: match parameters.channels.unwrap_or(2) {
			1 => Channels::Mono,
			2 => Channels::Stereo,
			4 => Channels::Quad,
			6 => Channels::Surround51,
			8 => Channels::Surround71,
			other => Channels::Custom(other as u64),
		},
		frame_size: 0,
		base: CodecState { extradata: Vec::new(), pts: None, dts: None, last_pts: None },
		samples: Samples::S16(Vec::new()),
		samples_consumed: 0,
		samples_produced: 0,
		buffer: Vec::new(),
	})
}

pub fn video_state(parameters: &Parameters) -> State {
	State::Video(VideoState {
		width: parameters.width.unwrap_or(0),
		height: parameters.height.unwrap_or(0),
		pixel: Pixel { depth: 8, format: video::PixelFormat::YUV420 },
		reorder: Vec::new(),
		has_keyframe: false,
		base: CodecState { extradata: Vec::new(), pts: None, dts: None, last_pts: None },
	})
}

pub fn default_codec_id(kind: StreamKind) -> CodecId {
	match kind {
		StreamKind::Audio => CodecId::new("pcm_s16le"),
		StreamKind::Video => CodecId::new("rawvideo"),
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
