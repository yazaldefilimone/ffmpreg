use crate::core::{Frame, Transform};
use crate::io::IoResult;

pub struct PeakLimiter {
	threshold: f32,
	release_coeff: f32,
	current_gain: f32,
}

impl PeakLimiter {
	pub fn new(threshold_db: f32) -> Self {
		let threshold = 10.0f32.powf(threshold_db / 20.0);
		Self { threshold, release_coeff: 0.9999, current_gain: 1.0 }
	}

	pub fn with_release(mut self, release_ms: f32, sample_rate: u32) -> Self {
		let release_samples = release_ms * sample_rate as f32 / 1000.0;
		self.release_coeff = (-1.0 / release_samples).exp();
		self
	}
}

impl Transform for PeakLimiter {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let samples = audio_frame.data.len() / 2;

			for i in 0..samples {
				let offset = i * 2;
				let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
				let sample_f = sample as f32 / 32768.0;

				let peak = sample_f.abs();
				let target_gain = if peak > self.threshold { self.threshold / peak } else { 1.0 };

				if target_gain < self.current_gain {
					self.current_gain = target_gain;
				} else {
					self.current_gain =
						self.current_gain * self.release_coeff + target_gain * (1.0 - self.release_coeff);
				}

				let limited = (sample_f * self.current_gain * 32767.0).clamp(-32768.0, 32767.0) as i16;
				let bytes = limited.to_le_bytes();
				audio_frame.data[offset] = bytes[0];
				audio_frame.data[offset + 1] = bytes[1];
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"peak_limiter"
	}
}
