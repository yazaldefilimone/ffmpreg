use super::bits::BitReader;
use super::header::{FrameHeader, MpegVersion};

#[derive(Debug, Clone, Copy, Default)]
pub struct GranuleChannel {
	pub part2_3_length: u16,
	pub big_values: u16,
	pub global_gain: u8,
	pub scalefac_compress: u16,
	pub window_switching: bool,
	pub block_type: u8,
	pub mixed_block: bool,
	pub table_select: [u8; 3],
	pub subblock_gain: [u8; 3],
	pub region0_count: u8,
	pub region1_count: u8,
	pub preflag: bool,
	pub scalefac_scale: bool,
	pub count1table_select: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Granule {
	pub channels: [GranuleChannel; 2],
}

#[derive(Debug, Clone)]
pub struct SideInfo {
	pub main_data_begin: u16,
	pub private_bits: u8,
	pub scfsi: [[bool; 4]; 2],
	pub granules: [Granule; 2],
}

impl SideInfo {
	pub fn parse(reader: &mut BitReader, header: &FrameHeader) -> Option<Self> {
		let channels = header.channels as usize;
		let is_mpeg1 = header.version == MpegVersion::Mpeg1;

		let main_data_begin =
			if is_mpeg1 { reader.read_bits(9)? as u16 } else { reader.read_bits(8)? as u16 };

		let private_bits = if is_mpeg1 {
			if channels == 1 { reader.read_bits(5)? as u8 } else { reader.read_bits(3)? as u8 }
		} else {
			if channels == 1 { reader.read_bits(1)? as u8 } else { reader.read_bits(2)? as u8 }
		};

		let mut scfsi = [[false; 4]; 2];
		if is_mpeg1 {
			for ch in 0..channels {
				for band in 0..4 {
					scfsi[ch][band] = reader.read_bits(1)? == 1;
				}
			}
		}

		let num_granules = if is_mpeg1 { 2 } else { 1 };
		let mut granules = [Granule::default(), Granule::default()];

		for gr in 0..num_granules {
			for ch in 0..channels {
				let gc = &mut granules[gr].channels[ch];

				gc.part2_3_length = reader.read_bits(12)? as u16;
				gc.big_values = reader.read_bits(9)? as u16;
				gc.global_gain = reader.read_bits(8)? as u8;

				gc.scalefac_compress =
					if is_mpeg1 { reader.read_bits(4)? as u16 } else { reader.read_bits(9)? as u16 };

				gc.window_switching = reader.read_bits(1)? == 1;

				if gc.window_switching {
					gc.block_type = reader.read_bits(2)? as u8;
					gc.mixed_block = reader.read_bits(1)? == 1;

					for i in 0..2 {
						gc.table_select[i] = reader.read_bits(5)? as u8;
					}

					for i in 0..3 {
						gc.subblock_gain[i] = reader.read_bits(3)? as u8;
					}

					if gc.block_type == 0 {
						return None;
					}

					if gc.block_type == 2 && !gc.mixed_block {
						gc.region0_count = 8;
					} else {
						gc.region0_count = 7;
					}
					gc.region1_count = 20 - gc.region0_count;
				} else {
					gc.block_type = 0;
					gc.mixed_block = false;

					for i in 0..3 {
						gc.table_select[i] = reader.read_bits(5)? as u8;
					}

					gc.region0_count = reader.read_bits(4)? as u8;
					gc.region1_count = reader.read_bits(3)? as u8;
				}

				if is_mpeg1 {
					gc.preflag = reader.read_bits(1)? == 1;
				}

				gc.scalefac_scale = reader.read_bits(1)? == 1;
				gc.count1table_select = reader.read_bits(1)? == 1;
			}
		}

		Some(Self { main_data_begin, private_bits, scfsi, granules })
	}
}
