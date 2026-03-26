use crate::core::{Metadata, Muxer, Packet, Stream};
use crate::io::Io;
use crate::message::Result;

pub struct RiffMuxer {
	io: Box<dyn Io>,
	metadata: Metadata,
	data_size_offset: u64,
	written: u64,
	streamed: bool,
}

impl RiffMuxer {
	pub fn new(io: Box<dyn Io>) -> Self {
		Self { io, metadata: Metadata::default(), data_size_offset: 0, written: 0, streamed: false }
	}

	fn write_header(&mut self, stream: &Stream) -> Result<()> {
		let p = &stream.parameters;

		let sample_rate = p.sample_rate.ok_or("missing sample_rate")?;
		let channels = p.channels.ok_or("missing channels")?;
		let (format_tag, bits_per_sample) = wav_format_from_codec(&p.codec.id)?;
		let bytes_per_sample = u32::from(bits_per_sample).div_ceil(8);
		let byte_rate = sample_rate * u32::from(channels) * bytes_per_sample;
		let block_align = u16::from(channels) * bits_per_sample.div_ceil(8);

		let info_chunk = build_info_chunk(&self.metadata);

		self.io.write_all(b"RIFF")?;
		self.io.write_all(&0u32.to_le_bytes())?;
		self.io.write_all(b"WAVE")?;
		self.io.write_all(b"fmt ")?;
		self.io.write_all(&(16u32).to_le_bytes())?;
		self.io.write_all(&format_tag.to_le_bytes())?;
		self.io.write_all(&(channels as u16).to_le_bytes())?;
		self.io.write_all(&sample_rate.to_le_bytes())?;
		self.io.write_all(&byte_rate.to_le_bytes())?;
		self.io.write_all(&block_align.to_le_bytes())?;
		self.io.write_all(&bits_per_sample.to_le_bytes())?;
		if let Some(chunk) = info_chunk {
			self.io.write_all(&chunk)?;
		}
		self.io.write_all(b"data")?;
		self.data_size_offset = self.io.size()?;
		self.io.write_all(&0u32.to_le_bytes())?;
		self.streamed = true;

		Ok(())
	}

	fn finalize(&mut self) -> Result<()> {
		let file_size = 36 + self.written;

		self.io.seek(4)?;
		self.io.write_all(&(file_size as u32).to_le_bytes())?;

		self.io.seek(self.data_size_offset)?;
		self.io.write_all(&(self.written as u32).to_le_bytes())?;

		Ok(())
	}
}

fn build_info_chunk(metadata: &Metadata) -> Option<Vec<u8>> {
	let mut info = Vec::new();

	push_info_entry(&mut info, b"INAM", metadata.title.as_deref());
	push_info_entry(&mut info, b"IART", metadata.artist.as_deref());
	push_info_entry(&mut info, b"IPRD", metadata.album.as_deref());
	push_info_entry(&mut info, b"ICMT", metadata.comment.as_deref());
	push_info_entry(&mut info, b"IGNR", metadata.genre.as_deref());
	push_info_entry(&mut info, b"ICRD", metadata.date.as_deref());

	let track = metadata.track_number.map(|value| value.to_string());
	push_info_entry(&mut info, b"ITRK", track.as_deref());

	if info.is_empty() {
		return None;
	}

	let mut chunk = Vec::new();
	chunk.extend_from_slice(b"LIST");
	chunk.extend_from_slice(&(info.len() as u32 + 4).to_le_bytes());
	chunk.extend_from_slice(b"INFO");
	chunk.extend_from_slice(&info);
	if (info.len() + 4) % 2 != 0 {
		chunk.push(0);
	}
	Some(chunk)
}

fn push_info_entry(out: &mut Vec<u8>, id: &[u8; 4], value: Option<&str>) {
	let Some(value) = value else {
		return;
	};
	if value.is_empty() {
		return;
	}

	let mut bytes = value.as_bytes().to_vec();
	bytes.push(0);

	out.extend_from_slice(id);
	out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
	out.extend_from_slice(&bytes);
	if bytes.len() % 2 != 0 {
		out.push(0);
	}
}

impl Muxer for RiffMuxer {
	fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
		self.metadata = metadata;
		Ok(())
	}

	fn metadata(&self) -> &Metadata {
		&self.metadata
	}

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
