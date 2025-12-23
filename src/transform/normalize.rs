use crate::core::{Frame, Transform};
use crate::io::IoResult;

pub struct Normalize {
	target_peak: f32,
}

impl Normalize {
	pub fn new(target_peak: f32) -> Self {
		Self { target_peak: target_peak.clamp(0.0, 1.0) }
	}

	pub fn default_peak() -> Self {
		Self::new(0.95)
	}
}

impl Transform for Normalize {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let samples = audio_frame.data.len() / 2;
			if samples == 0 {
				return Ok(frame);
			}

			let mut max_amplitude: i16 = 0;
			for i in 0..samples {
				let offset = i * 2;
				let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
				max_amplitude = max_amplitude.max(sample.abs());
			}

			if max_amplitude == 0 {
				return Ok(frame);
			}

			let target = (32767.0 * self.target_peak) as i16;
			let scale = target as f32 / max_amplitude as f32;

			for i in 0..samples {
				let offset = i * 2;
				let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
				let normalized = (sample as f32 * scale).clamp(-32768.0, 32767.0) as i16;
				let bytes = normalized.to_le_bytes();
				audio_frame.data[offset] = bytes[0];
				audio_frame.data[offset + 1] = bytes[1];
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"normalize"
	}
}
