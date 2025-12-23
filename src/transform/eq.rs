use crate::core::{Frame, Transform};
use crate::io::IoResult;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
	LowShelf,
	HighShelf,
	Peaking,
}

#[derive(Debug, Clone)]
pub struct EqBand {
	pub filter_type: FilterType,
	pub frequency: f32,
	pub gain_db: f32,
	pub q: f32,
}

impl EqBand {
	pub fn low_shelf(frequency: f32, gain_db: f32) -> Self {
		Self { filter_type: FilterType::LowShelf, frequency, gain_db, q: 0.707 }
	}

	pub fn high_shelf(frequency: f32, gain_db: f32) -> Self {
		Self { filter_type: FilterType::HighShelf, frequency, gain_db, q: 0.707 }
	}

	pub fn peaking(frequency: f32, gain_db: f32, q: f32) -> Self {
		Self { filter_type: FilterType::Peaking, frequency, gain_db, q }
	}
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

pub struct Equalizer {
	bands: Vec<EqBand>,
	coeffs: Vec<BiquadCoeffs>,
	states: Vec<Vec<BiquadState>>,
	sample_rate: u32,
	initialized: bool,
}

impl Equalizer {
	pub fn new(bands: Vec<EqBand>) -> Self {
		Self { bands, coeffs: Vec::new(), states: Vec::new(), sample_rate: 44100, initialized: false }
	}

	pub fn three_band(bass_db: f32, mid_db: f32, treble_db: f32) -> Self {
		Self::new(vec![
			EqBand::low_shelf(200.0, bass_db),
			EqBand::peaking(1000.0, mid_db, 1.0),
			EqBand::high_shelf(4000.0, treble_db),
		])
	}

	fn calculate_coeffs(&mut self, sample_rate: u32) {
		self.coeffs.clear();
		self.sample_rate = sample_rate;

		for band in &self.bands {
			let omega = 2.0 * PI * band.frequency / sample_rate as f32;
			let sin_omega = omega.sin();
			let cos_omega = omega.cos();
			let alpha = sin_omega / (2.0 * band.q);
			let a = 10.0f32.powf(band.gain_db / 40.0);

			let (b0, b1, b2, a0, a1, a2) = match band.filter_type {
				FilterType::LowShelf => {
					let sqrt_a = a.sqrt();
					let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
					let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
					let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
					let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
					let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
					let a2 = (a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
					(b0, b1, b2, a0, a1, a2)
				}
				FilterType::HighShelf => {
					let sqrt_a = a.sqrt();
					let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
					let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
					let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
					let a0 = (a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
					let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
					let a2 = (a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
					(b0, b1, b2, a0, a1, a2)
				}
				FilterType::Peaking => {
					let b0 = 1.0 + alpha * a;
					let b1 = -2.0 * cos_omega;
					let b2 = 1.0 - alpha * a;
					let a0 = 1.0 + alpha / a;
					let a1 = -2.0 * cos_omega;
					let a2 = 1.0 - alpha / a;
					(b0, b1, b2, a0, a1, a2)
				}
			};

			self.coeffs.push(BiquadCoeffs {
				b0: b0 / a0,
				b1: b1 / a0,
				b2: b2 / a0,
				a1: a1 / a0,
				a2: a2 / a0,
			});
		}
	}

	fn process_sample(&mut self, sample: f32, channel: usize) -> f32 {
		let mut output = sample;

		for (band_idx, coeffs) in self.coeffs.iter().enumerate() {
			let state = &mut self.states[channel][band_idx];

			let y = coeffs.b0 * output + coeffs.b1 * state.x1 + coeffs.b2 * state.x2
				- coeffs.a1 * state.y1
				- coeffs.a2 * state.y2;

			state.x2 = state.x1;
			state.x1 = output;
			state.y2 = state.y1;
			state.y1 = y;

			output = y;
		}

		output
	}
}

impl Transform for Equalizer {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			if !self.initialized || self.sample_rate != audio_frame.sample_rate {
				self.calculate_coeffs(audio_frame.sample_rate);
				self.states = (0..audio_frame.channels as usize)
					.map(|_| (0..self.bands.len()).map(|_| BiquadState::default()).collect())
					.collect();
				self.initialized = true;
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
		"equalizer"
	}
}
