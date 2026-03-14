pub use crate::container::wav::demuxer::WavDemuxer;
pub use crate::container::wav::muxer::WavMuxer;
use crate::core::frame::{BitDepth, Channels, SampleRate};
use crate::{codecs, core};

#[derive(Debug, Clone, Copy)]
pub struct WavFormat {
	pub channels: Channels,
	pub sample_rate: SampleRate,
	pub bit_depth: BitDepth,
	pub format_code: u16,
}

impl Default for WavFormat {
	fn default() -> Self {
		// defaut is pcm_16
		Self {
			channels: Channels::Stereo,
			sample_rate: SampleRate::SR44_1K,
			bit_depth: BitDepth::Bit32,
			format_code: 3,
		}
	}
}

impl WavFormat {
	pub fn new_for_codec(codec: &str) -> Result<Self, String> {
		match codec {
			"pcm_s16le" => Ok(Self { bit_depth: BitDepth::Bit16, format_code: 1, ..Self::default() }),
			"pcm_s24le" => Ok(Self { bit_depth: BitDepth::Bit24, ..Self::default() }),
			"pcm_f32le" => Ok(Self { bit_depth: BitDepth::Bit32, format_code: 3, ..Self::default() }),
			_ => Err(format!("wav codec '{}' is not supported", codec)),
		}
	}

	pub fn bytes_per_sample(&self) -> usize {
		self.bit_depth.bytes()
	}

	pub fn bytes_per_frame(&self) -> usize {
		self.bytes_per_sample() * self.channels.count() as usize
	}

	pub fn byte_rate(&self) -> u32 {
		self
			.sample_rate
			.value()
			.saturating_mul(self.channels.count() as u32)
			.saturating_mul(self.bytes_per_sample() as u32)
	}

	pub fn block_align(&self) -> u16 {
		self.channels.count() as u16 * (self.bit_depth.bytes() as u16)
	}

	pub fn to_codec_string(&self) -> &'static str {
		match self.bit_depth.bits() {
			16 => "pcm_s16le",
			24 => "pcm_s24le",
			32 => "pcm_f32le",
			_ => "pcm_f32le",
		}
	}

	pub fn to_codec_id(&self) -> core::CodecId {
		match self.bit_depth.bits() {
			16 => codecs::PCM_S16LE,
			24 => codecs::PCM_S24LE,
			32 => codecs::PCM_F32LE,
			_ => codecs::PCM_F32LE,
		}
	}

	pub fn apply_codec(&mut self, codec: &str) -> Result<(), String> {
		match codec {
			"pcm_s16le" => self.bit_depth = BitDepth::Bit16,
			"pcm_s24le" => self.bit_depth = BitDepth::Bit24,
			"pcm_f32le" => {
				self.bit_depth = BitDepth::Bit32;
				self.format_code = 3;
			}
			_ => return Err(format!("wav codec '{}' is not supported", codec)),
		}
		Ok(())
	}
}
