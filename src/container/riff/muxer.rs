use crate::core::{Muxer, Packet, Stream};
use crate::io::Io;
use crate::message::Result;

pub struct RiffMuxer {
	io: Box<dyn Io>,
	written: u64,
	streamed: bool,
}

impl RiffMuxer {
	pub fn new(io: Box<dyn Io>) -> Self {
		Self { io, written: 0, streamed: false }
	}

	fn write_header(&mut self, stream: &Stream) -> Result<()> {
		let p = &stream.parameters;

		let sample_rate = p.sample_rate.ok_or("missing sample_rate")?;
		let channels = p.channels.ok_or("missing channels")?;
		let (format_tag, bits_per_sample) = wav_format_from_codec(&p.codec.id)?;
		let bytes_per_sample = u32::from(bits_per_sample).div_ceil(8);
		let byte_rate = sample_rate * u32::from(channels) * bytes_per_sample;
		let block_align = u16::from(channels) * bits_per_sample.div_ceil(8);

		let mut header = [0u8; 44];

		header[0..4].copy_from_slice(b"RIFF");
		header[8..12].copy_from_slice(b"WAVE");
		header[12..16].copy_from_slice(b"fmt ");
		header[16..20].copy_from_slice(&(16u32).to_le_bytes());
		header[20..22].copy_from_slice(&format_tag.to_le_bytes());
		header[22..24].copy_from_slice(&(channels as u16).to_le_bytes());
		header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
		header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
		header[32..34].copy_from_slice(&block_align.to_le_bytes());
		header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());
		header[36..40].copy_from_slice(b"data");

		self.io.write_all(&header)?;
		self.streamed = true;

		Ok(())
	}

	fn finalize(&mut self) -> Result<()> {
		let file_size = 36 + self.written;

		self.io.seek(4)?;
		self.io.write_all(&(file_size as u32).to_le_bytes())?;

		self.io.seek(40)?;
		self.io.write_all(&(self.written as u32).to_le_bytes())?;

		Ok(())
	}
}

impl Muxer for RiffMuxer {
	fn add(&mut self, stream: &Stream) -> Result<usize> {
		if self.streamed {
			return Err("wav supports only one stream".into());
		}
		self.write_header(stream)?;
		Ok(0)
	}

	fn write(&mut self, packet: Packet) -> Result<()> {
		self.io.write_all(&packet.data)?;
		self.written += packet.data.len() as u64;
		Ok(())
	}

	fn finish(&mut self) -> Result<()> {
		self.finalize()
	}
}

fn wav_format_from_codec(codec: &str) -> Result<(u16, u16)> {
	match codec {
		"pcm_u8" => Ok((1, 8)),
		"pcm_s16" | "pcm_s16le" => Ok((1, 16)),
		"pcm_s24" | "pcm_s24le" => Ok((1, 24)),
		"pcm_s32" | "pcm_s32le" => Ok((1, 32)),
		"pcm_f32" | "pcm_f32le" => Ok((3, 32)),
		"pcm_f64" | "pcm_f64le" => Ok((3, 64)),
		_ => Err("unsupported wav codec".into()),
	}
}
