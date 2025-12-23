use crate::core::{Frame, Transform};
use crate::io::IoResult;

pub struct RmsLimiter {
	threshold_db: f32,
	window_samples: usize,
	release_coeff: f32,
	current_gain: f32,
	rms_buffer: Vec<f32>,
	buffer_pos: usize,
}

impl RmsLimiter {
	pub fn new(threshold_db: f32, window_ms: f32, sample_rate: u32) -> Self {
		let window_samples = (window_ms * sample_rate as f32 / 1000.0) as usize;
		Self {
			threshold_db,
			window_samples: window_samples.max(1),
			release_coeff: 0.9995,
			current_gain: 1.0,
			rms_buffer: vec![0.0; window_samples.max(1)],
			buffer_pos: 0,
		}
	}

	fn db_to_linear(db: f32) -> f32 {
		10.0f32.powf(db / 20.0)
	}

	fn calculate_rms(&self) -> f32 {
		let sum: f32 = self.rms_buffer.iter().sum();
		(sum / self.rms_buffer.len() as f32).sqrt()
	}
}

impl Transform for RmsLimiter {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let samples = audio_frame.data.len() / 2;
			let threshold_linear = Self::db_to_linear(self.threshold_db);

			for i in 0..samples {
				let offset = i * 2;
				let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
				let sample_f = sample as f32 / 32768.0;

				self.rms_buffer[self.buffer_pos] = sample_f * sample_f;
				self.buffer_pos = (self.buffer_pos + 1) % self.window_samples;

				let rms = self.calculate_rms();
				let target_gain = if rms > threshold_linear { threshold_linear / rms } else { 1.0 };

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
		"rms_limiter"
	}
}
