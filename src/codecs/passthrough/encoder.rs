use crate::core;
use crate::{core::*, message::Result};
#[derive(Debug, Clone)]
pub struct Encoder {
	stream_id: StreamId,
	kind: StreamKind,
}

impl Encoder {
	pub fn new(stream_id: StreamId, kind: StreamKind) -> Self {
		Self { stream_id, kind }
	}
}

impl core::Encoder for Encoder {
	fn create_state(&self, parameters: &Parameters) -> State {
		match self.kind {
			StreamKind::Audio => super::audio_state(parameters),
			_ => super::video_state(parameters),
		}
	}

	fn encode(&self, state: &mut State, frame: Frame) -> Result<EncodeOut<State>> {
		let packet = match frame {
			Frame::Audio(frame) => Packet {
				stream_id: self.stream_id,
				data: frame.data,
				pts: frame.pts,
				dts: frame.pts,
				duration: None,
			},
			Frame::Video(frame) => Packet {
				stream_id: self.stream_id,
				data: frame.data,
				pts: frame.pts,
				dts: frame.pts,
				duration: None,
			},
		};

		Ok(EncodeOut { value: vec![packet], state: state.clone() })
	}

	fn flush(&self, state: &mut State) -> Result<EncodeOut<State>> {
		Ok(EncodeOut { value: Vec::new(), state: state.clone() })
	}
}
