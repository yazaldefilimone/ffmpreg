use super::format::WavFormat;
use super::header::WavHeader;
use crate::container::constants::CHUNK_SIZE_LIMIT;
use crate::core::frame::Channels;
use crate::core::packet::Packet;
use crate::core::time::{Time, Timestamp};
use crate::core::track::{AudioFormat, Metadata, TrackFormat};
use crate::core::{Demuxer, Track, Tracks};
use crate::io::{BinaryRead, MediaRead};
use crate::{error, message::Result};

pub struct WavDemuxer<R: MediaRead> {
	reader: R,
	format: WavFormat,
	track: Track,
	metadata: Metadata,
	remaining_bytes: u64,
	samples_read: u64,
	packet_count: u64,
	time: Time,
	read_buf: Vec<u8>,
}

impl<R: MediaRead> WavDemuxer<R> {
	pub fn new(mut reader: R) -> Result<Self> {
		let (header, metadata, remaining_bytes) = Self::read_wav_and_find_data(&mut reader)?;
		header.validate()?;

		let format = header.to_format();
		let time = Time::new(1, header.sample_rate)?;
		let block_align = format.block_align() as u64;
		let max_chunk = (CHUNK_SIZE_LIMIT as u64 / block_align) * block_align;
		let codec_in = format.to_codec_id();

		let audio_format = AudioFormat {
			channels: format.channels,
			bit_depth: format.bit_depth,
			sample_rate: format.sample_rate,
		};

		let track = Track {
			id: 0,
			codec_in,
			codec_out: codec_in,
			timestamp: Timestamp::zero(time),
			format: TrackFormat::Audio(audio_format),
		};

		Ok(Self {
			reader,
			format,
			track,
			metadata,
			remaining_bytes,
			samples_read: 0,
			packet_count: 0,
			time,
			read_buf: vec![0u8; max_chunk as usize],
		})
	}

	fn read_wav_and_find_data(reader: &mut R) -> Result<(WavHeader, Metadata, u64)> {
		Self::check_fourcc_eq(reader, b"RIFF")?;
		let _file_size = reader.read_u32_le()?;
		Self::check_fourcc_eq(reader, b"WAVE")?;

		let mut header = WavHeader::default();
		let mut metadata = Metadata::default();

		loop {
			let mut chunk_id = [0u8; 4];
			reader.read_exact(&mut chunk_id)?;
			let chunk_size = reader.read_u32_le()? as u64;

			match &chunk_id {
				b"fmt " => Self::read_fmt_chunk(reader, chunk_size, &mut header)?,
				b"LIST" => Self::read_list_chunk(reader, chunk_size, &mut metadata)?,
				b"data" => return Ok((header, metadata, chunk_size)),
				_ => Self::skip_bytes(reader, chunk_size)?,
			}
		}
	}

	fn read_fmt_chunk(reader: &mut R, chunk_size: u64, header: &mut WavHeader) -> Result<()> {
		if chunk_size < 16 {
			return Err(error!("fmt chunk too small"));
		}

		header.format_code = reader.read_u16_le()?;
		header.channels = Channels::from_count(reader.read_u16_le()? as u8);
		header.sample_rate = reader.read_u32_le()?;
		header.byte_rate = reader.read_u32_le()?;
		header.block_align = reader.read_u16_le()?;
		header.bits_per_sample = reader.read_u16_le()?;

		if chunk_size > 16 {
			Self::skip_bytes(reader, chunk_size - 16)?;
		}
		Ok(())
	}

	fn read_list_chunk(reader: &mut R, chunk_size: u64, metadata: &mut Metadata) -> Result<()> {
		if chunk_size < 4 {
			return Ok(());
		}

		let mut form_type = [0u8; 4];
		reader.read_exact(&mut form_type)?;
		if form_type != *b"INFO" {
			return Self::skip_bytes(reader, chunk_size - 4);
		}

		let mut pos = 4u64;
		while pos + 8 <= chunk_size {
			let mut id = [0u8; 4];
			reader.read_exact(&mut id)?;
			let size = reader.read_u32_le()? as u64;
			pos += 8;

			let data = Self::read_bytes(reader, size)?;
			pos += size;

			let value = String::from_utf8_lossy(&data).trim_end_matches('\0').to_string();
			Self::apply_wav_tag(metadata, &id, value);

			if size % 2 == 1 {
				reader.read_u8()?;
				pos += 1;
			}
		}
		Ok(())
	}

	/// Map WAV RIFF INFO tags to standard metadata field names.
	fn apply_wav_tag(metadata: &mut Metadata, id: &[u8; 4], value: String) {
		let key = match id {
			b"IART" => "artist",
			b"INAM" => "title",
			b"ICOM" => "comment",
			b"ICOP" => "copyright",
			b"ISFT" => "software",
			b"IGNR" => "genre",
			b"ITRK" => "track",
			_ => return,
		};
		metadata.set(key, value);
	}

	fn check_fourcc_eq(reader: &mut R, expected: &[u8; 4]) -> Result<()> {
		let mut buf = [0u8; 4];
		reader.read_exact(&mut buf)?;
		if buf != *expected {
			return Err(error!(
				"expected {}, found {}",
				String::from_utf8_lossy(expected),
				String::from_utf8_lossy(&buf)
			));
		}
		Ok(())
	}

	fn read_bytes(reader: &mut R, size: u64) -> Result<Vec<u8>> {
		let mut buf = vec![0u8; size as usize];
		reader.read_exact(&mut buf)?;
		Ok(buf)
	}

	fn skip_bytes(reader: &mut R, mut size: u64) -> Result<()> {
		const BUF_SIZE: usize = 8192;
		let mut buf = [0u8; BUF_SIZE];
		while size > 0 {
			let read_size = std::cmp::min(size, BUF_SIZE as u64) as usize;
			reader.read_exact(&mut buf[..read_size])?;
			size -= read_size as u64;
		}
		Ok(())
	}

	pub fn read_packet(&mut self) -> Result<Option<Packet>> {
		if self.remaining_bytes == 0 {
			return Ok(None);
		}

		let read_len = std::cmp::min(self.remaining_bytes as usize, self.read_buf.len());
		let bytes_read = self.reader.read(&mut self.read_buf[..read_len])?;
		if bytes_read == 0 {
			return Ok(None);
		}

		let data = self.read_buf[..bytes_read].to_vec();
		self.remaining_bytes -= bytes_read as u64;

		let packet = Packet::new(data, 0, self.time).with_pts(self.samples_read as i64);

		self.samples_read += (bytes_read / self.format.bytes_per_frame()) as u64;
		self.packet_count += 1;

		Ok(Some(packet))
	}

	pub fn format(&self) -> WavFormat {
		self.format
	}

	pub fn metadata(&self) -> &Metadata {
		&self.metadata
	}
}

impl<R: MediaRead> Demuxer for WavDemuxer<R> {
	fn read(&mut self) -> Result<Option<Packet>> {
		self.read_packet()
	}

	fn seek(&mut self, _time: f64) -> Result<()> {
		Err(error!("seek not implemented for wav"))
	}

	fn duration(&self) -> Option<f64> {
		let bytes_sample = self.format.bytes_per_sample() as u64;

		let total_samples = self.samples_read + self.remaining_bytes / bytes_sample;

		let rate = self.format.sample_rate.value() as f64;

		Some(total_samples as f64 / rate)
	}

	fn tracks(&self) -> Tracks {
		Tracks::new(vec![self.track])
	}

	fn metadata(&self) -> &Metadata {
		&self.metadata
	}
}
