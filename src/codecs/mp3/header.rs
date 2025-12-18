use super::bits::BitReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
	Mpeg1,
	Mpeg2,
	Mpeg25,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
	Layer1,
	Layer2,
	Layer3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
	Stereo,
	JointStereo,
	DualChannel,
	Mono,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
	pub version: MpegVersion,
	pub layer: Layer,
	pub crc_protection: bool,
	pub bitrate: u32,
	pub sample_rate: u32,
	pub padding: bool,
	pub private_bit: bool,
	pub channel_mode: ChannelMode,
	pub mode_extension: u8,
	pub copyright: bool,
	pub original: bool,
	pub emphasis: u8,
	pub frame_size: usize,
	pub channels: u8,
}

const BITRATE_TABLE: [[[u32; 15]; 3]; 2] = [
	[
		[0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448],
		[0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384],
		[0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320],
	],
	[
		[0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256],
		[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
		[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
	],
];

const SAMPLE_RATE_TABLE: [[u32; 3]; 3] =
	[[44100, 48000, 32000], [22050, 24000, 16000], [11025, 12000, 8000]];

impl FrameHeader {
	pub fn parse(data: &[u8]) -> Option<Self> {
		if data.len() < 4 {
			return None;
		}

		let mut reader = BitReader::new(data);

		let sync = reader.read_bits(11)?;
		if sync != 0x7FF {
			return None;
		}

		let version_bits = reader.read_bits(2)?;
		let version = match version_bits {
			0 => MpegVersion::Mpeg25,
			2 => MpegVersion::Mpeg2,
			3 => MpegVersion::Mpeg1,
			_ => return None,
		};

		let layer_bits = reader.read_bits(2)?;
		let layer = match layer_bits {
			1 => Layer::Layer3,
			2 => Layer::Layer2,
			3 => Layer::Layer1,
			_ => return None,
		};

		let crc_protection = reader.read_bits(1)? == 0;
		let bitrate_index = reader.read_bits(4)? as usize;
		let sample_rate_index = reader.read_bits(2)? as usize;
		let padding = reader.read_bits(1)? == 1;
		let private_bit = reader.read_bits(1)? == 1;

		let channel_mode_bits = reader.read_bits(2)?;
		let channel_mode = match channel_mode_bits {
			0 => ChannelMode::Stereo,
			1 => ChannelMode::JointStereo,
			2 => ChannelMode::DualChannel,
			3 => ChannelMode::Mono,
			_ => return None,
		};

		let mode_extension = reader.read_bits(2)? as u8;
		let copyright = reader.read_bits(1)? == 1;
		let original = reader.read_bits(1)? == 1;
		let emphasis = reader.read_bits(2)? as u8;

		if bitrate_index == 0 || bitrate_index == 15 || sample_rate_index == 3 {
			return None;
		}

		let version_idx = match version {
			MpegVersion::Mpeg1 => 0,
			_ => 1,
		};

		let layer_idx = match layer {
			Layer::Layer1 => 0,
			Layer::Layer2 => 1,
			Layer::Layer3 => 2,
		};

		let sr_version_idx = match version {
			MpegVersion::Mpeg1 => 0,
			MpegVersion::Mpeg2 => 1,
			MpegVersion::Mpeg25 => 2,
		};

		let bitrate = BITRATE_TABLE[version_idx][layer_idx][bitrate_index] * 1000;
		let sample_rate = SAMPLE_RATE_TABLE[sr_version_idx][sample_rate_index];

		let frame_size = match layer {
			Layer::Layer1 => (12 * bitrate / sample_rate + if padding { 1 } else { 0 }) * 4,
			Layer::Layer2 | Layer::Layer3 => {
				let samples_per_frame = match (version, layer) {
					(MpegVersion::Mpeg1, Layer::Layer3) => 1152,
					(_, Layer::Layer3) => 576,
					_ => 1152,
				};
				samples_per_frame * bitrate / 8 / sample_rate + if padding { 1 } else { 0 }
			}
		} as usize;

		let channels = if channel_mode == ChannelMode::Mono { 1 } else { 2 };

		Some(Self {
			version,
			layer,
			crc_protection,
			bitrate,
			sample_rate,
			padding,
			private_bit,
			channel_mode,
			mode_extension,
			copyright,
			original,
			emphasis,
			frame_size,
			channels,
		})
	}

	pub fn samples_per_frame(&self) -> usize {
		match (self.version, self.layer) {
			(MpegVersion::Mpeg1, Layer::Layer1) => 384,
			(MpegVersion::Mpeg1, _) => 1152,
			(_, Layer::Layer1) => 384,
			(_, Layer::Layer3) => 576,
			(_, _) => 1152,
		}
	}

	pub fn side_info_size(&self) -> usize {
		match (self.version, self.channel_mode) {
			(MpegVersion::Mpeg1, ChannelMode::Mono) => 17,
			(MpegVersion::Mpeg1, _) => 32,
			(_, ChannelMode::Mono) => 9,
			(_, _) => 17,
		}
	}

	pub fn is_intensity_stereo(&self) -> bool {
		self.channel_mode == ChannelMode::JointStereo && (self.mode_extension & 0x01) != 0
	}

	pub fn is_ms_stereo(&self) -> bool {
		self.channel_mode == ChannelMode::JointStereo && (self.mode_extension & 0x02) != 0
	}
}

pub fn find_sync(data: &[u8]) -> Option<usize> {
	for i in 0..data.len().saturating_sub(1) {
		if data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
			if FrameHeader::parse(&data[i..]).is_some() {
				return Some(i);
			}
		}
	}
	None
}
