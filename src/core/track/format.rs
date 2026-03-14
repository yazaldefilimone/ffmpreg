use crate::{
	container::wav::WavFormat,
	container::yuv::YuvFormat,
	core::frame::{BitDepth, Channels, SampleRate},
};
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
	pub channels: Channels,
	pub bit_depth: BitDepth,
	pub sample_rate: SampleRate,
}

impl AudioFormat {
	pub fn bytes_per_frame(&self) -> usize {
		let bytes_per_sample = self.bit_depth.bytes() as usize;
		let channels = self.channels.count() as usize;
		bytes_per_sample.saturating_mul(channels)
	}

	pub fn block_align(&self) -> u16 {
		let bytes_per_sample = self.bit_depth.bytes() as u16;
		let channels = self.channels.count() as u16;
		bytes_per_sample.saturating_mul(channels)
	}

	pub fn byte_rate(&self) -> u32 {
		let channels = self.channels.count() as u32;
		let bit_depth = self.bit_depth.bytes() as u32;
		let value = self.sample_rate.value();
		value.saturating_mul(channels).saturating_mul(bit_depth)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoFormat {
	pub width: u32,
	pub height: u32,
	pub pixel: Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
	pub depth: BitDepth,
	pub color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
	YUV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackFormat {
	Audio(AudioFormat),
	Video(VideoFormat),
}

impl TrackFormat {
	pub fn audio(&self) -> Option<&AudioFormat> {
		match self {
			Self::Audio(audio) => Some(audio),
			_ => None,
		}
	}
	pub fn video(&self) -> Option<&VideoFormat> {
		match self {
			Self::Video(video) => Some(video),
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Format {
	Wav(WavFormat),
	Flac(AudioFormat),
	Ogg(AudioFormat),
	Vorbis(AudioFormat),
	Mp3(AudioFormat),
	Aac(AudioFormat),
	Yuv(YuvFormat),
}

impl Default for AudioFormat {
	fn default() -> Self {
		Self {
			channels: Channels::Stereo,
			bit_depth: BitDepth::Bit16,
			sample_rate: SampleRate::SR44_1K,
		}
	}
}

impl Format {
	pub fn wav() -> Self {
		Self::Wav(WavFormat::default())
	}

	pub fn flac() -> Self {
		Self::Flac(AudioFormat::default())
	}

	pub fn ogg() -> Self {
		Self::Ogg(AudioFormat::default())
	}

	pub fn vorbis() -> Self {
		Self::Vorbis(AudioFormat::default())
	}

	pub fn mp3() -> Self {
		Self::Mp3(AudioFormat::default())
	}

	pub fn aac() -> Self {
		Self::Aac(AudioFormat::default())
	}

	pub fn yuv() -> Self {
		Self::Yuv(YuvFormat::default())
	}

	pub fn from_container(container: crate::container::ContainerId) -> crate::message::Result<Self> {
		match container.name {
			"wav" => Ok(Self::wav()),
			"flac" => Ok(Self::flac()),
			"ogg" | "oga" => Ok(Self::ogg()),
			"mp3" => Ok(Self::mp3()),
			"yuv" => Ok(Self::yuv()),
			_ => Err(crate::error!("unsupported container '{}'", container.name)),
		}
	}

	pub fn apply_codec(&mut self, codec: &str) -> crate::message::Result<()> {
		match self {
			Format::Wav(wav) => wav.apply_codec(codec).map_err(|e| crate::error!("{}", e)),
			_ => Err(crate::error!("codec override not supported for '{}'", self)),
		}
	}

	pub fn inherit_audio(&mut self, audio: &AudioFormat) {
		match self {
			Format::Wav(wav) => {
				wav.sample_rate = audio.sample_rate;
				wav.channels = audio.channels;
			}
			Format::Flac(fmt)
			| Format::Ogg(fmt)
			| Format::Vorbis(fmt)
			| Format::Mp3(fmt)
			| Format::Aac(fmt) => {
				fmt.sample_rate = audio.sample_rate;
				fmt.channels = audio.channels;
			}
			Format::Yuv(_) => {}
		}
	}
}

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Format::Wav(_) => write!(f, "wav"),
			Format::Flac(_) => write!(f, "flac"),
			Format::Ogg(_) => write!(f, "ogg"),
			Format::Vorbis(_) => write!(f, "vorbis"),
			Format::Mp3(_) => write!(f, "mp3"),
			Format::Aac(_) => write!(f, "aac"),
			Format::Yuv(_) => write!(f, "yuv"),
		}
	}
}

impl fmt::Display for TrackFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Audio(_) => write!(f, "audio"),
			Self::Video(_) => write!(f, "video"),
		}
	}
}
