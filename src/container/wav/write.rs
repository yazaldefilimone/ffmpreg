use super::WavFormat;
use crate::core::{Muxer, Packet};
use std::io::{Result, Seek, SeekFrom, Write};

pub struct WavWriter<W: Write + Seek> {
	writer: W,
	data_size: u32,
}

impl<W: Write + Seek> WavWriter<W> {
	pub fn new(mut writer: W, format: WavFormat) -> Result<Self> {
		Self::write_header(&mut writer, format, 0)?;
		Ok(Self { writer, data_size: 0 })
	}

	fn write_header(writer: &mut W, format: WavFormat, data_size: u32) -> Result<()> {
		let byte_rate = format.sample_rate * format.bytes_per_frame() as u32;
		let block_align = format.bytes_per_frame() as u16;

		writer.write_all(b"RIFF")?;
		writer.write_all(&(36 + data_size).to_le_bytes())?;
		writer.write_all(b"WAVE")?;

		writer.write_all(b"fmt ")?;
		writer.write_all(&16u32.to_le_bytes())?;
		writer.write_all(&1u16.to_le_bytes())?;
		writer.write_all(&(format.channels as u16).to_le_bytes())?;
		writer.write_all(&format.sample_rate.to_le_bytes())?;
		writer.write_all(&byte_rate.to_le_bytes())?;
		writer.write_all(&block_align.to_le_bytes())?;
		writer.write_all(&format.bit_depth.to_le_bytes())?;

		writer.write_all(b"data")?;
		writer.write_all(&data_size.to_le_bytes())?;

		Ok(())
	}
}

impl<W: Write + Seek> Muxer for WavWriter<W> {
	fn write_packet(&mut self, packet: Packet) -> Result<()> {
		self.writer.write_all(&packet.data)?;
		self.data_size += packet.size() as u32;
		Ok(())
	}

	fn finalize(&mut self) -> Result<()> {
		let current_pos = self.writer.stream_position()?;
		self.writer.seek(SeekFrom::Start(4))?;
		self.writer.write_all(&(36 + self.data_size).to_le_bytes())?;
		self.writer.seek(SeekFrom::Start(40))?;
		self.writer.write_all(&self.data_size.to_le_bytes())?;
		self.writer.seek(SeekFrom::Start(current_pos))?;
		Ok(())
	}
}
