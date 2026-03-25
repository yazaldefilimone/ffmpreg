use crate::Message;
use crate::core::{CodecId, Demuxer, Packet, Stream, StreamId, StreamSet, Time};
use crate::io::Io;
use crate::message::Result;

pub struct RiffDemuxer {
	io: Box<dyn Io>,
	streams: StreamSet,
	data_start: u64,
	size: u64,
	remaining: u64,
	byte_rate: u32,
	block_align: u16,
}

impl RiffDemuxer {
	pub fn new(mut io: Box<dyn Io>) -> Result<Self> {
		let mut riff_header = [0u8; 12];
		read_exact(&mut *io, &mut riff_header)?;
		if &riff_header[0..4] != b"RIFF" || &riff_header[8..12] != b"WAVE" {
			return Err(Message::Container("invalid wav header"));
		}

		let mut offset = 12u64;
		let mut channels = None;
		let mut sample_rate = None;
		let mut byte_rate = None;
		let mut block_align = None;
		let mut bits_per_sample = None;
		let mut codec = None;
		let mut data_start = None;
		let mut size = None;

		loop {
			let mut chunk_header = [0u8; 8];
			if read_exact_or_eof(&mut *io, &mut chunk_header)? == 0 {
				break;
			}
			offset += 8;

			let chunk_id = &chunk_header[0..4];
			let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap()) as u64;
			let padded_size = chunk_size + (chunk_size & 1);

			if chunk_id == b"fmt " {
				if chunk_size < 16 {
					return Err(Message::Container("invalid wav fmt chunk"));
				}

				let mut fmt = vec![0u8; chunk_size as usize];
				read_exact(&mut *io, &mut fmt)?;
				offset += chunk_size;

				let format_tag = u16::from_le_bytes(fmt[0..2].try_into().unwrap());
				let fmt_channels = u16::from_le_bytes(fmt[2..4].try_into().unwrap());
				let fmt_sample_rate = u32::from_le_bytes(fmt[4..8].try_into().unwrap());
				let fmt_byte_rate = u32::from_le_bytes(fmt[8..12].try_into().unwrap());
				let fmt_block_align = u16::from_le_bytes(fmt[12..14].try_into().unwrap());
				let fmt_bits_per_sample = u16::from_le_bytes(fmt[14..16].try_into().unwrap());

				let resolved_codec = resolve_codec_id(format_tag, fmt_bits_per_sample, &fmt)?;

				if fmt_channels == 0 {
					return Err(Message::Container("invalid wav channel count"));
				}
				if fmt_sample_rate == 0 {
					return Err(Message::Container("invalid wav sample rate"));
				}
				if fmt_block_align == 0 {
					return Err(Message::Container("invalid wav block align"));
				}
				if fmt_byte_rate == 0 {
					return Err(Message::Container("invalid wav byte rate"));
				}

				channels = Some(fmt_channels);
				sample_rate = Some(fmt_sample_rate);
				byte_rate = Some(fmt_byte_rate);
				block_align = Some(fmt_block_align);
				bits_per_sample = Some(fmt_bits_per_sample);
				codec = Some(resolved_codec);

				if padded_size > chunk_size {
					skip_exact(&mut *io, padded_size - chunk_size)?;
					offset += padded_size - chunk_size;
				}
				continue;
			}

			if chunk_id == b"data" {
				data_start = Some(offset);
				size = Some(chunk_size);
				break;
			}

			skip_exact(&mut *io, padded_size)?;
			offset += padded_size;
		}
		let fmt_err = || Message::Container("wav fmt chunk not found");
		let data_err = || Message::Container("wav data chunk not found");

		let channels = channels.ok_or_else(fmt_err)?;
		let sample_rate = sample_rate.ok_or_else(fmt_err)?;
		let byte_rate = byte_rate.ok_or_else(fmt_err)?;
		let block_align = block_align.ok_or_else(fmt_err)?;
		let bits_per_sample = bits_per_sample.ok_or_else(fmt_err)?;
		let codec = codec.ok_or_else(fmt_err)?;

		let data_start = data_start.ok_or_else(data_err)?;
		let size = size.ok_or_else(data_err)?;

		if bits_per_sample == 0 {
			return Err(Message::Container("invalid wav bits per sample"));
		}

		let bytes_per_sample = bits_per_sample.div_ceil(8);
		let expected_block_align = u32::from(channels) * u32::from(bytes_per_sample);
		if u32::from(block_align) != expected_block_align {
			return Err(Message::Container("inconsistent wav block align"));
		}

		let expected_byte_rate = sample_rate * expected_block_align;
		if byte_rate != expected_byte_rate {
			return Err(Message::Container("inconsistent wav byte rate"));
		}

		let stream = Stream::audio(StreamId(0), sample_rate, channels as u8, codec);

		let mut streams = StreamSet::default();
		streams.add(stream);

		Ok(Self { io, streams, data_start, size, remaining: size, byte_rate, block_align })
	}
}

impl Demuxer for RiffDemuxer {
	fn read(&mut self) -> Result<Option<Packet>> {
		if self.remaining == 0 {
			return Ok(None);
		}

		let align = usize::from(self.block_align.max(1));
		let mut size = 4096.min(self.remaining as usize);
		if size > align {
			size -= size % align;
		}
		if size == 0 {
			size = self.remaining as usize;
		}
		let mut data = vec![0u8; size];
		let n = self.io.read(&mut data)?;

		if n == 0 {
			return Ok(None);
		}

		data.truncate(n);
		self.remaining -= n as u64;

		Ok(Some(Packet { stream_id: StreamId(0), data, pts: None, dts: None, duration: None }))
	}

	fn streams(&self) -> &StreamSet {
		&self.streams
	}

	fn seek(&mut self, time: f64) -> Result<()> {
		if !time.is_finite() || time < 0.0 {
			return Err("invalid seek time".into());
		}

		if self.byte_rate == 0 {
			return Err("invalid wav byte rate".into());
		}

		let raw_offset = (time * self.byte_rate as f64).floor() as u64;
		let align = u64::from(self.block_align.max(1));
		let aligned_offset = (raw_offset / align) * align;
		let clamped_offset = aligned_offset.min(self.size);

		self.io.seek(self.data_start + clamped_offset)?;
		self.remaining = self.size.saturating_sub(clamped_offset);
		Ok(())
	}
	fn duration(&self) -> Time {
		let stream = self.streams.get(StreamId(0)).unwrap();
		let tb = stream.time_base;

		if self.block_align == 0 {
			return Time::zero(tb);
		}
		let samples = self.size / self.block_align as u64;

		Time::new(samples, tb)
	}
}

fn read_exact(io: &mut dyn Io, mut buf: &mut [u8]) -> Result<()> {
	while !buf.is_empty() {
		let n = io.read(buf)?;
		if n == 0 {
			return Err(Message::Container("unexpected end of file"));
		}
		let (_, rest) = buf.split_at_mut(n);
		buf = rest;
	}
	Ok(())
}

fn read_exact_or_eof(io: &mut dyn Io, mut buf: &mut [u8]) -> Result<usize> {
	let mut total = 0;
	while !buf.is_empty() {
		let n = io.read(buf)?;
		if n == 0 {
			if total == 0 {
				return Ok(0);
			}
			return Err(Message::Container("unexpected end of file"));
		}
		total += n;
		let (_, rest) = buf.split_at_mut(n);
		buf = rest;
	}
	Ok(total)
}

fn skip_exact(io: &mut dyn Io, mut size: u64) -> Result<()> {
	let mut buf = [0u8; 1024];
	while size > 0 {
		let chunk = (buf.len() as u64).min(size) as usize;
		let n = io.read(&mut buf[..chunk])?;
		if n == 0 {
			return Err(Message::Container("unexpected end of file"));
		}
		size -= n as u64;
	}
	Ok(())
}

fn resolve_codec_id(format_tag: u16, bits_per_sample: u16, fmt: &[u8]) -> Result<CodecId> {
	let resolved_tag =
		if format_tag == 0xFFFE { resolve_extensible_subformat(fmt)? } else { format_tag };

	match resolved_tag {
		1 => match bits_per_sample {
			8 => Ok(CodecId::new("pcm_u8")),
			16 => Ok(CodecId::new("pcm_s16le")),
			24 => Ok(CodecId::new("pcm_s24le")),
			32 => Ok(CodecId::new("pcm_s32le")),
			64 => Ok(CodecId::new("pcm_s64le")),
			_ => Err(Message::Container("unsupported wav pcm bit depth")),
		},
		3 => match bits_per_sample {
			32 => Ok(CodecId::new("pcm_f32le")),
			64 => Ok(CodecId::new("pcm_f64le")),
			_ => Err(Message::Container("unsupported wav float bit depth")),
		},
		6 => Ok(CodecId::new("pcm_alaw")),
		7 => Ok(CodecId::new("pcm_mulaw")),
		_ => Err(Message::Container("unsupported wav format")),
	}
}

fn resolve_extensible_subformat(fmt: &[u8]) -> Result<u16> {
	if fmt.len() < 40 {
		return Err(Message::Container("invalid wav extensible chunk"));
	}

	let cb_size = u16::from_le_bytes(fmt[16..18].try_into().unwrap());
	if cb_size < 22 {
		return Err(Message::Container("invalid wav extensible size"));
	}

	Ok(u16::from_le_bytes(fmt[24..26].try_into().unwrap()))
}
