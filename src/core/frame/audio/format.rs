#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
	Mono,
	Stereo,
	Quad,
	Surround51,
	Surround71,
	Custom(u64),
}

impl ChannelLayout {
	pub const fn count(self) -> u8 {
		match self {
			ChannelLayout::Mono => 1,
			ChannelLayout::Stereo => 2,
			ChannelLayout::Quad => 4,
			ChannelLayout::Surround51 => 6,
			ChannelLayout::Surround71 => 8,
			ChannelLayout::Custom(c) => c.count_ones() as u8,
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
	pub channels: ChannelLayout,
	pub sample_format: SampleFormat,
	pub codec_extradata: Vec<u8>,
}
