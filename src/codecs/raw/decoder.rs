use crate::core;
use crate::{core::*, message::Result};

#[derive(Debug, Clone)]
pub struct Decoder {
	kind: StreamKind,
}

impl Decoder {
	pub fn new(kind: StreamKind) -> Self {
		Self { kind }
	}
}

impl core::Decoder for Decoder {
	fn create_state(&self, parameters: &Parameters) -> State {
		match self.kind {
			StreamKind::Audio => super::audio_state(parameters),
			_ => super::video_state(parameters),
		}
	}

	fn decode(&self, state: &mut State, packet: Packet) -> Result<DecodeOut<State>> {
		let frame = match self.kind {
			StreamKind::Audio => {
				let (sample_rate, channels, format, layout) = match state {
					State::Audio(audio) => (audio.sample_rate, audio.channels, audio.format, audio.layout),
					State::Video(_) => {
						(48_000, Channels::Stereo, SampleFormat::S16, SampleLayout::Interleaved)
					}
				};
				let mut frame = AudioFrame::new(packet.data.clone(), sample_rate, channels, format, layout);
				frame.pts = packet.pts;
				Frame::Audio(frame)
			}
			_ => {
				let (width, height, pixel) = match state {
					State::Video(video) => (video.width, video.height, video.pixel.clone()),
					State::Audio(_) => (0, 0, video::Pixel { depth: 8, format: video::PixelFormat::YUV420 }),
				};
				Frame::Video(VideoFrame {
					data: packet.data.clone(),
					width,
					height,
					pixel,
					keyframe: Keyframe::Key,
					pts: packet.pts,
				})
			}
		};

		Ok(DecodeOut { value: vec![frame], state: state.clone() })
	}

	fn flush(&self, state: &mut State) -> Result<DecodeOut<State>> {
		Ok(DecodeOut { value: Vec::new(), state: state.clone() })
	}
}
