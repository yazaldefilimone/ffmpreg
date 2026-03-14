use serde::{Deserialize, Serialize};

use crate::core::resampler::pcm::PcmResampler;
use crate::core::{time::Timestamp, track};
use crate::message;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitDepth {
	Bit16,
	Bit24,
	Bit32,
	Custom(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channels {
	Mono,
	Stereo,
	Quad,
	Surround,      // 5.1
	SevenPointOne, // 7.1
	Custom(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleRate {
	SR44_1K,
	SR48K,
	SR96K,
	Custom(u32),
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
	pub data: Vec<u8>,
	pub sample_rate: SampleRate,
	pub channels: Channels,
	pub bit_depth: BitDepth,
	pub nb_samples: usize,
	pub pts: Timestamp,
}

impl BitDepth {
	pub const fn bits(self) -> u8 {
		match self {
			Self::Bit16 => 16,
			Self::Bit24 => 24,
			Self::Bit32 => 32,
			Self::Custom(b) => b,
		}
	}

	pub const fn bytes(self) -> usize {
		(self.bits() as usize) / 8
	}

	pub const fn from_bits(bits: u8) -> Option<Self> {
		match bits {
			16 => Some(Self::Bit16),
			24 => Some(Self::Bit24),
			32 => Some(Self::Bit32),
			_ => None,
		}
	}

	pub const fn from_bits_any(bits: u8) -> Self {
		match bits {
			16 => Self::Bit16,
			24 => Self::Bit24,
			32 => Self::Bit32,
			bits => Self::Custom(bits),
		}
	}
}

impl Channels {
	pub const fn count(self) -> u8 {
		match self {
			Self::Mono => 1,
			Self::Stereo => 2,
			Self::Quad => 4,
			Self::Surround => 6,
			Self::SevenPointOne => 8,
			Self::Custom(c) => c,
		}
	}

	pub const fn from_count(count: u8) -> Self {
		match count {
			1 => Self::Mono,
			2 => Self::Stereo,
			4 => Self::Quad,
			6 => Self::Surround,
			8 => Self::SevenPointOne,
			count => Self::Custom(count),
		}
	}
}

impl SampleRate {
	pub const fn value(self) -> u32 {
		match self {
			Self::SR44_1K => 44_100,
			Self::SR48K => 48_000,
			Self::SR96K => 96_000,
			Self::Custom(simple_rate) => simple_rate,
		}
	}

	pub const fn from_value(simple_rate: u32) -> Self {
		match simple_rate {
			44_100 => Self::SR44_1K,
			48_000 => Self::SR48K,
			96_000 => Self::SR96K,
			simple_rate => Self::Custom(simple_rate),
		}
	}
}

impl AudioFrame {
	pub fn new(
		data: Vec<u8>,
		sample_rate: SampleRate,
		channels: Channels,
		bit_depth: BitDepth,
	) -> Self {
		let nb_samples = data.len() / (channels.count() as usize * bit_depth.bytes());
		Self { data, sample_rate, channels, bit_depth, nb_samples, pts: Timestamp::default() }
	}

	#[inline(always)]
	pub const fn frame_size(&self) -> usize {
		self.nb_samples * self.channels.count() as usize * self.bit_depth.bytes()
	}

	pub fn with_nb_samples(mut self, nb_samples: usize) -> Self {
		self.nb_samples = nb_samples;
		self
	}

	pub fn with_bit_depth(mut self, bit_depth: BitDepth) -> Self {
		self.bit_depth = bit_depth;
		self
	}

	pub fn with_pts(mut self, pts: Timestamp) -> Self {
		self.pts = pts;
		self
	}

	pub const fn duration_seconds(&self) -> f64 {
		self.nb_samples as f64 / self.sample_rate.value() as f64
	}

	pub fn audio_format(&self) -> track::AudioFormat {
		track::AudioFormat {
			sample_rate: self.sample_rate,
			channels: self.channels,
			bit_depth: self.bit_depth,
		}
	}

	pub fn as_samples(&self) -> message::Result<Vec<f32>> {
		let pcm = PcmResampler::new(self.bit_depth, self.bit_depth);
		pcm.decode(&self.data)
	}

	pub fn set_samples(&mut self, samples: &[f32]) -> message::Result<()> {
		let pcm = PcmResampler::new(self.bit_depth, self.bit_depth);
		self.data = pcm.encode(samples)?;
		Ok(())
	}
}

impl fmt::Display for BitDepth {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Bit16 => write!(f, "16-bit"),
			Self::Bit24 => write!(f, "24-bit"),
			Self::Bit32 => write!(f, "32-bit"),
			Self::Custom(b) => write!(f, "{}-bit", b),
		}
	}
}

impl fmt::Display for Channels {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			Self::Mono => "mono",
			Self::Stereo => "stereo",
			Self::Quad => "quad",
			Self::Surround => "5.1",
			Self::SevenPointOne => "7.1",
			Self::Custom(numbers) => {
				if *numbers > 1 {
					return write!(f, "{} channels", numbers);
				}
				return write!(f, "{} channel", numbers);
			}
		};
		write!(f, "{}", s)
	}
}

impl fmt::Display for SampleRate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::SR44_1K => write!(f, "44.1kHz"),
			Self::SR48K => write!(f, "48kHz"),
			Self::SR96K => write!(f, "96kHz"),
			Self::Custom(sample_rate) => write!(f, "{}Hz", sample_rate),
		}
	}
}
