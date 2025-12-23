use crate::container::WavFormat;
use crate::core::{Decoder, Encoder, Frame, FrameAudio, Packet, Timebase};
use crate::io::IoResult;

const MS_ADAPT_TABLE: [i16; 16] =
	[230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230];

const MS_COEF1: [i16; 7] = [256, 512, 0, 192, 240, 460, 392];
const MS_COEF2: [i16; 7] = [0, -256, 0, 64, 0, -208, -232];

#[derive(Debug, Clone)]
struct MsAdpcmChannelState {
	sample1: i16,
	sample2: i16,
	delta: i16,
	coef1: i16,
	coef2: i16,
}

impl Default for MsAdpcmChannelState {
	fn default() -> Self {
		Self { sample1: 0, sample2: 0, delta: 16, coef1: 256, coef2: 0 }
	}
}

impl MsAdpcmChannelState {
	fn decode_sample(&mut self, nibble: i8) -> i16 {
		let prediction =
			((self.sample1 as i32 * self.coef1 as i32) + (self.sample2 as i32 * self.coef2 as i32)) >> 8;

		let signed_nibble = nibble as i32;
		let new_sample = prediction + (signed_nibble * self.delta as i32);
		let clamped = new_sample.clamp(-32768, 32767) as i16;

		self.sample2 = self.sample1;
		self.sample1 = clamped;

		let adapt_index = (nibble & 0x0F) as usize;
		let new_delta = (self.delta as i32 * MS_ADAPT_TABLE[adapt_index] as i32) >> 8;
		self.delta = new_delta.max(16) as i16;

		clamped
	}

	fn encode_sample(&mut self, sample: i16) -> i8 {
		let prediction =
			((self.sample1 as i32 * self.coef1 as i32) + (self.sample2 as i32 * self.coef2 as i32)) >> 8;

		let diff = sample as i32 - prediction;
		let nibble = (diff / self.delta as i32).clamp(-8, 7) as i8;

		let reconstructed = prediction + (nibble as i32 * self.delta as i32);
		let clamped = reconstructed.clamp(-32768, 32767) as i16;

		self.sample2 = self.sample1;
		self.sample1 = clamped;

		let adapt_index = (nibble & 0x0F) as usize;
		let new_delta = (self.delta as i32 * MS_ADAPT_TABLE[adapt_index] as i32) >> 8;
		self.delta = new_delta.max(16) as i16;

		nibble
	}
}

pub struct MsAdpcmDecoder {
	format: WavFormat,
}

impl MsAdpcmDecoder {
	pub fn new(format: WavFormat, _block_size: usize) -> Self {
		Self { format }
	}

	fn decode_block(&self, data: &[u8], channels: usize) -> Vec<i16> {
		if data.len() < 7 * channels {
			return Vec::new();
		}

		let mut states: Vec<MsAdpcmChannelState> =
			(0..channels).map(|_| MsAdpcmChannelState::default()).collect();

		let mut pos = 0;

		for ch in 0..channels {
			let predictor_idx = data[pos].min(6) as usize;
			states[ch].coef1 = MS_COEF1[predictor_idx];
			states[ch].coef2 = MS_COEF2[predictor_idx];
			pos += 1;
		}

		for ch in 0..channels {
			states[ch].delta = i16::from_le_bytes([data[pos], data[pos + 1]]);
			pos += 2;
		}

		for ch in 0..channels {
			states[ch].sample1 = i16::from_le_bytes([data[pos], data[pos + 1]]);
			pos += 2;
		}

		for ch in 0..channels {
			states[ch].sample2 = i16::from_le_bytes([data[pos], data[pos + 1]]);
			pos += 2;
		}

		let mut output = Vec::new();

		for ch in 0..channels {
			output.push(states[ch].sample2);
		}
		for ch in 0..channels {
			output.push(states[ch].sample1);
		}

		while pos < data.len() {
			let byte = data[pos];
			pos += 1;

			let high_nibble = ((byte >> 4) as i8).wrapping_shl(4).wrapping_shr(4);
			let low_nibble = ((byte & 0x0F) as i8).wrapping_shl(4).wrapping_shr(4);

			if channels == 1 {
				output.push(states[0].decode_sample(high_nibble));
				output.push(states[0].decode_sample(low_nibble));
			} else {
				output.push(states[0].decode_sample(high_nibble));
				output.push(states[1].decode_sample(low_nibble));
			}
		}

		output
	}
}

impl Decoder for MsAdpcmDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let channels = self.format.channels as usize;
		let samples = self.decode_block(&packet.data, channels);

		let mut output = Vec::with_capacity(samples.len() * 2);
		for sample in &samples {
			output.extend_from_slice(&sample.to_le_bytes());
		}

		let nb_samples = samples.len() / channels;
		let audio = FrameAudio::new(output, self.format.sample_rate, self.format.channels)
			.with_nb_samples(nb_samples);
		let frame = Frame::new_audio(audio, packet.timebase, packet.stream_index).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}

pub struct MsAdpcmEncoder {
	timebase: Timebase,
	channels: u8,
}

impl MsAdpcmEncoder {
	pub fn new(timebase: Timebase, channels: u8, _block_size: usize) -> Self {
		Self { timebase, channels }
	}

	fn find_best_predictor(&self, samples: &[i16]) -> usize {
		if samples.len() < 2 {
			return 0;
		}

		let mut best_idx = 0;
		let mut min_error = i64::MAX;

		for (idx, (&c1, &c2)) in MS_COEF1.iter().zip(MS_COEF2.iter()).enumerate() {
			let mut error: i64 = 0;
			for i in 2..samples.len().min(10) {
				let prediction =
					((samples[i - 1] as i32 * c1 as i32) + (samples[i - 2] as i32 * c2 as i32)) >> 8;
				let diff = samples[i] as i32 - prediction;
				error += (diff as i64).pow(2);
			}
			if error < min_error {
				min_error = error;
				best_idx = idx;
			}
		}

		best_idx
	}

	fn encode_block(&self, samples: &[i16], channels: usize) -> Vec<u8> {
		let mut output = Vec::new();

		let channel_samples: Vec<Vec<i16>> = (0..channels)
			.map(|ch| samples.iter().skip(ch).step_by(channels).copied().collect())
			.collect();

		let predictors: Vec<usize> =
			channel_samples.iter().map(|cs| self.find_best_predictor(cs)).collect();

		for &pred_idx in &predictors {
			output.push(pred_idx as u8);
		}

		let mut states: Vec<MsAdpcmChannelState> = predictors
			.iter()
			.map(|&idx| {
				let mut state = MsAdpcmChannelState::default();
				state.coef1 = MS_COEF1[idx];
				state.coef2 = MS_COEF2[idx];
				state
			})
			.collect();

		for ch in 0..channels {
			if channel_samples[ch].len() >= 2 {
				let s1 = channel_samples[ch][0];
				let s2 = channel_samples[ch][1];
				let diff = (s2 as i32 - s1 as i32).unsigned_abs() as i16;
				states[ch].delta = diff.max(16);
			}
			output.extend_from_slice(&states[ch].delta.to_le_bytes());
		}

		for ch in 0..channels {
			let sample = channel_samples[ch].get(1).copied().unwrap_or(0);
			states[ch].sample1 = sample;
			output.extend_from_slice(&sample.to_le_bytes());
		}

		for ch in 0..channels {
			let sample = channel_samples[ch].first().copied().unwrap_or(0);
			states[ch].sample2 = sample;
			output.extend_from_slice(&sample.to_le_bytes());
		}

		let samples_per_channel = channel_samples[0].len();

		if channels == 1 {
			let mut i = 2;
			while i + 1 < samples_per_channel {
				let nibble1 = states[0].encode_sample(channel_samples[0][i]);
				let nibble2 = states[0].encode_sample(channel_samples[0][i + 1]);
				output.push(((nibble1 & 0x0F) << 4) as u8 | (nibble2 & 0x0F) as u8);
				i += 2;
			}
			if i < samples_per_channel {
				let nibble = states[0].encode_sample(channel_samples[0][i]);
				output.push(((nibble & 0x0F) << 4) as u8);
			}
		} else {
			for i in 2..samples_per_channel {
				let nibble_l = states[0].encode_sample(channel_samples[0][i]);
				let nibble_r = states[1].encode_sample(channel_samples[1].get(i).copied().unwrap_or(0));
				output.push(((nibble_l & 0x0F) << 4) as u8 | (nibble_r & 0x0F) as u8);
			}
		}

		output
	}
}

impl Encoder for MsAdpcmEncoder {
	fn encode(&mut self, frame: Frame) -> IoResult<Option<Packet>> {
		let data_bytes = match &frame.data {
			crate::core::FrameData::Audio(audio) => &audio.data,
			crate::core::FrameData::Video(video) => &video.data,
		};

		let samples: Vec<i16> =
			data_bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();

		let channels = self.channels as usize;
		let output = self.encode_block(&samples, channels);

		let packet = Packet::new(output, frame.stream_index, self.timebase).with_pts(frame.pts);
		Ok(Some(packet))
	}

	fn flush(&mut self) -> IoResult<Option<Packet>> {
		Ok(None)
	}
}
