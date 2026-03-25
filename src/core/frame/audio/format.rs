#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Channels {
	Mono,
	#[default]
	Stereo,
	Quad,
	Surround51,
	Surround71,
	Custom(u8),
}

impl Channels {
	pub const fn count(self) -> u8 {
		match self {
			Channels::Mono => 1,
			Channels::Stereo => 2,
			Channels::Quad => 4,
			Channels::Surround51 => 6,
			Channels::Surround71 => 8,
			Channels::Custom(c) => c,
		}
	}

	pub const fn from_value(value: u8) -> Self {
		match value {
			1 => Channels::Mono,
			2 => Channels::Stereo,
			4 => Channels::Quad,
			6 => Channels::Surround51,
			8 => Channels::Surround71,
			_ => Channels::Custom(value),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
	S16,
	S32,
	F32,
	F64,
}

impl SampleFormat {
	pub const fn bytes(self) -> usize {
		match self {
			SampleFormat::S16 => 2,
			SampleFormat::S32 => 4,
			SampleFormat::F32 => 4,
			SampleFormat::F64 => 8,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleRate {
	SR44_1K,
	SR48K,
	SR96K,
	Custom(u32),
}

impl SampleRate {
	pub const fn value(self) -> u32 {
		match self {
			SampleRate::SR44_1K => 44_100,
			SampleRate::SR48K => 48_000,
			SampleRate::SR96K => 96_000,
			SampleRate::Custom(rate) => rate,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AudioParams {
	pub sample_rate: SampleRate,
	pub channels: Channels,
	pub sample_format: SampleFormat,
	pub codec_extradata: Vec<u8>,
}
