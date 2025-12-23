use crate::core::{Frame, Transform};
use crate::io::IoResult;
use std::f32::consts::PI;

pub struct Lowpass {
	cutoff: f32,
	q: f32,
	coeffs: Option<BiquadCoeffs>,
	states: Vec<BiquadState>,
	sample_rate: u32,
}

struct BiquadCoeffs {
	b0: f32,
	b1: f32,
	b2: f32,
	a1: f32,
	a2: f32,
}

struct BiquadState {
	x1: f32,
	x2: f32,
	y1: f32,
	y2: f32,
}

impl Default for BiquadState {
	fn default() -> Self {
		Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
	}
}

impl Lowpass {
	pub fn new(cutoff: f32) -> Self {
		Self { cutoff, q: 0.707, coeffs: None, states: Vec::new(), sample_rate: 0 }
	}

	pub fn with_q(mut self, q: f32) -> Self {
		self.q = q;
		self
	}

	fn calculate_coeffs(&mut self, sample_rate: u32) {
		self.sample_rate = sample_rate;

		let omega = 2.0 * PI * self.cutoff / sample_rate as f32;
		let sin_omega = omega.sin();
		let cos_omega = omega.cos();
		let alpha = sin_omega / (2.0 * self.q);

		let b0 = (1.0 - cos_omega) / 2.0;
		let b1 = 1.0 - cos_omega;
		let b2 = (1.0 - cos_omega) / 2.0;
		let a0 = 1.0 + alpha;
		let a1 = -2.0 * cos_omega;
		let a2 = 1.0 - alpha;

		self.coeffs =
			Some(BiquadCoeffs { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 });
	}

	fn process_sample(&mut self, sample: f32, channel: usize) -> f32 {
		let coeffs = self.coeffs.as_ref().unwrap();
		let state = &mut self.states[channel];

		let y = coeffs.b0 * sample + coeffs.b1 * state.x1 + coeffs.b2 * state.x2
			- coeffs.a1 * state.y1
			- coeffs.a2 * state.y2;

		state.x2 = state.x1;
		state.x1 = sample;
		state.y2 = state.y1;
		state.y1 = y;

		y
	}
}

impl Transform for Lowpass {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			if self.sample_rate != audio_frame.sample_rate {
				self.calculate_coeffs(audio_frame.sample_rate);
			}

			if self.states.len() != audio_frame.channels as usize {
				self.states = (0..audio_frame.channels as usize).map(|_| BiquadState::default()).collect();
			}

			let channels = audio_frame.channels as usize;
			let samples_per_channel = audio_frame.nb_samples;

			for i in 0..samples_per_channel {
				for ch in 0..channels {
					let offset = (i * channels + ch) * 2;
					let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
					let sample_f = sample as f32 / 32768.0;

					let processed = self.process_sample(sample_f, ch);
					let output = (processed * 32767.0).clamp(-32768.0, 32767.0) as i16;

					let bytes = output.to_le_bytes();
					audio_frame.data[offset] = bytes[0];
					audio_frame.data[offset + 1] = bytes[1];
				}
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"lowpass"
	}
}
