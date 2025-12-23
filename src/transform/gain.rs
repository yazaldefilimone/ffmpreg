use crate::core::{Frame, Transform};
use crate::io::IoResult;

pub struct Gain {
	factor: f32,
}

impl Gain {
	pub fn new(factor: f32) -> Self {
		Self { factor }
	}
}

impl Transform for Gain {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let samples = audio_frame.data.len() / 2;
			for i in 0..samples {
				let offset = i * 2;
				let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
				let amplified = (sample as f32 * self.factor).clamp(-32768.0, 32767.0) as i16;
				let bytes = amplified.to_le_bytes();
				audio_frame.data[offset] = bytes[0];
				audio_frame.data[offset + 1] = bytes[1];
			}
		}
		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"gain"
	}
}
