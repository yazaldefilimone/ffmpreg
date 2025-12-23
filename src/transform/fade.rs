use crate::core::{Frame, Transform};
use crate::io::IoResult;

pub struct FadeIn {
	duration_samples: usize,
	current_sample: usize,
}

impl FadeIn {
	pub fn new(duration_ms: f32, sample_rate: u32) -> Self {
		let duration_samples = (duration_ms * sample_rate as f32 / 1000.0) as usize;
		Self { duration_samples, current_sample: 0 }
	}
}

impl Transform for FadeIn {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let channels = audio_frame.channels as usize;
			let samples_per_channel = audio_frame.nb_samples;

			for i in 0..samples_per_channel {
				let gain = if self.current_sample < self.duration_samples {
					self.current_sample as f32 / self.duration_samples as f32
				} else {
					1.0
				};

				for ch in 0..channels {
					let offset = (i * channels + ch) * 2;
					let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
					let faded = (sample as f32 * gain).clamp(-32768.0, 32767.0) as i16;

					let bytes = faded.to_le_bytes();
					audio_frame.data[offset] = bytes[0];
					audio_frame.data[offset + 1] = bytes[1];
				}

				self.current_sample += 1;
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"fade_in"
	}
}

pub struct FadeOut {
	duration_samples: usize,
	total_samples: usize,
	current_sample: usize,
}

impl FadeOut {
	pub fn new(duration_ms: f32, total_duration_ms: f32, sample_rate: u32) -> Self {
		let duration_samples = (duration_ms * sample_rate as f32 / 1000.0) as usize;
		let total_samples = (total_duration_ms * sample_rate as f32 / 1000.0) as usize;
		Self { duration_samples, total_samples, current_sample: 0 }
	}

	pub fn from_sample_count(duration_samples: usize, total_samples: usize) -> Self {
		Self { duration_samples, total_samples, current_sample: 0 }
	}
}

impl Transform for FadeOut {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if let Some(audio_frame) = frame.audio_mut() {
			let channels = audio_frame.channels as usize;
			let samples_per_channel = audio_frame.nb_samples;
			let fade_start = self.total_samples.saturating_sub(self.duration_samples);

			for i in 0..samples_per_channel {
				let gain = if self.current_sample >= fade_start {
					let fade_pos = self.current_sample - fade_start;
					1.0 - (fade_pos as f32 / self.duration_samples as f32).min(1.0)
				} else {
					1.0
				};

				for ch in 0..channels {
					let offset = (i * channels + ch) * 2;
					let sample = i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
					let faded = (sample as f32 * gain).clamp(-32768.0, 32767.0) as i16;

					let bytes = faded.to_le_bytes();
					audio_frame.data[offset] = bytes[0];
					audio_frame.data[offset + 1] = bytes[1];
				}

				self.current_sample += 1;
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"fade_out"
	}
}

pub struct Crossfade {
	duration_samples: usize,
	current_sample: usize,
	crossfade_buffer: Vec<i16>,
	buffer_pos: usize,
	in_crossfade: bool,
}

impl Crossfade {
	pub fn new(duration_ms: f32, sample_rate: u32, channels: u8) -> Self {
		let duration_samples = (duration_ms * sample_rate as f32 / 1000.0) as usize;
		let buffer_size = duration_samples * channels as usize;
		Self {
			duration_samples,
			current_sample: 0,
			crossfade_buffer: vec![0i16; buffer_size],
			buffer_pos: 0,
			in_crossfade: false,
		}
	}

	pub fn start_crossfade(&mut self) {
		self.in_crossfade = true;
		self.buffer_pos = 0;
	}

	pub fn feed_previous(&mut self, frame: &Frame) {
		if let Some(audio_frame) = frame.audio() {
			let channels = audio_frame.channels as usize;
			let samples: Vec<i16> =
				audio_frame.data.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

			let start_sample = samples.len().saturating_sub(self.duration_samples * channels);
			for (i, &sample) in samples[start_sample..].iter().enumerate() {
				if self.buffer_pos + i < self.crossfade_buffer.len() {
					self.crossfade_buffer[self.buffer_pos + i] = sample;
				}
			}
		}
	}
}

impl Transform for Crossfade {
	fn apply(&mut self, mut frame: Frame) -> IoResult<Frame> {
		if !self.in_crossfade {
			return Ok(frame);
		}

		if let Some(audio_frame) = frame.audio_mut() {
			let channels = audio_frame.channels as usize;
			let samples_per_channel = audio_frame.nb_samples;

			for i in 0..samples_per_channel {
				if self.current_sample >= self.duration_samples {
					self.in_crossfade = false;
					break;
				}

				let fade_out = 1.0 - (self.current_sample as f32 / self.duration_samples as f32);
				let fade_in = self.current_sample as f32 / self.duration_samples as f32;

				for ch in 0..channels {
					let offset = (i * channels + ch) * 2;
					let buffer_idx = self.current_sample * channels + ch;

					let new_sample =
						i16::from_le_bytes([audio_frame.data[offset], audio_frame.data[offset + 1]]);
					let old_sample = if buffer_idx < self.crossfade_buffer.len() {
						self.crossfade_buffer[buffer_idx]
					} else {
						0
					};

					let mixed = (old_sample as f32 * fade_out + new_sample as f32 * fade_in)
						.clamp(-32768.0, 32767.0) as i16;

					let bytes = mixed.to_le_bytes();
					audio_frame.data[offset] = bytes[0];
					audio_frame.data[offset + 1] = bytes[1];
				}

				self.current_sample += 1;
			}
		}

		Ok(frame)
	}

	fn name(&self) -> &'static str {
		"crossfade"
	}
}
