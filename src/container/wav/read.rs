use super::WavFormat;
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{IoError, IoResult, MediaRead, ReadPrimitives};

pub struct WavReader<R: MediaRead> {
	reader: R,
	format: WavFormat,
	timebase: Timebase,
	data_remaining: u64,
	packet_count: u64,
}

impl<R: MediaRead> WavReader<R> {
	pub fn new(mut reader: R) -> IoResult<Self> {
		let format = Self::read_header(&mut reader)?;
		let (data_size, _) = Self::find_data_chunk(&mut reader)?;

		Ok(Self {
			reader,
			format,
			timebase: Timebase::new(1, format.sample_rate),
			data_remaining: data_size,
			packet_count: 0,
		})
	}

	pub fn format(&self) -> WavFormat {
		self.format
	}

	fn read_header(reader: &mut R) -> IoResult<WavFormat> {
		let mut buf = [0u8; 12];
		reader.read_exact(&mut buf)?;

		if &buf[0..4] != b"RIFF" {
			return Err(IoError::invalid_data("not a RIFF file"));
		}

		if &buf[8..12] != b"WAVE" {
			return Err(IoError::invalid_data("not a WAVE file"));
		}

		let channels;
		let sample_rate;
		let bit_depth;

		loop {
			let mut chunk_header = [0u8; 8];
			reader.read_exact(&mut chunk_header)?;

			let chunk_id = &chunk_header[0..4];
			let chunk_size =
				u32::from_le_bytes([chunk_header[4], chunk_header[5], chunk_header[6], chunk_header[7]])
					as usize;

			if chunk_id == b"fmt " {
				let mut fmt_buf = vec![0u8; chunk_size];
				reader.read_exact(&mut fmt_buf)?;

				if chunk_size < 16 {
					return Err(IoError::invalid_data("fmt chunk too small"));
				}

				channels = u16::from_le_bytes([fmt_buf[2], fmt_buf[3]]) as u8;
				sample_rate = u32::from_le_bytes([fmt_buf[4], fmt_buf[5], fmt_buf[6], fmt_buf[7]]);
				bit_depth = u16::from_le_bytes([fmt_buf[14], fmt_buf[15]]);

				if bit_depth != 16 {
					return Err(IoError::invalid_data("only 16-bit PCM supported"));
				}

				break;
			} else {
				let mut skip = vec![0u8; chunk_size];
				reader.read_exact(&mut skip)?;
			}
		}

		Ok(WavFormat { channels, sample_rate, bit_depth })
	}

	fn find_data_chunk(reader: &mut R) -> IoResult<(u64, u64)> {
		let mut buf = [0u8; 8];
		loop {
			reader.read_exact(&mut buf)?;
			if &buf[0..4] == b"data" {
				let size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as u64;
				return Ok((size, 0));
			}
		}
	}
}

impl<R: MediaRead> Demuxer for WavReader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		if self.data_remaining == 0 {
			return Ok(None);
		}

		let frame_size = 4096.min(self.data_remaining as usize);
		let mut buf = vec![0u8; frame_size];
		let read = self.reader.read(&mut buf)?;

		if read == 0 {
			return Ok(None);
		}

		buf.truncate(read);
		self.data_remaining -= read as u64;

		let pts = self.packet_count * read as u64 / self.format.bytes_per_frame() as u64;
		self.packet_count += 1;

		Ok(Some(Packet::new(buf, 0, self.timebase).with_pts(pts as i64)))
	}

	fn stream_count(&self) -> usize {
		1
	}
}
