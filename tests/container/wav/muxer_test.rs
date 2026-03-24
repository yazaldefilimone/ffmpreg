use ffmpreg::container::wav::muxer::WavMuxer;
use ffmpreg::core::{CodecId, Muxer, Packet, Stream, StreamId};
use ffmpreg::io::Io;

#[test]
fn wav_muxer_writes_valid_header_and_payload() {
	let io = SharedIo::default();
	let mut muxer = WavMuxer::new(Box::new(io.clone()));

	let stream = Stream::audio(StreamId(0), 44_100, 2, CodecId::new("pcm_s16le"));
	muxer.add(&stream).unwrap();
	muxer.write(Packet::new(StreamId(0), vec![1, 2, 3, 4])).unwrap();
	muxer.finish().unwrap();

	let data = io.snapshot();
	assert_eq!(&data[0..4], b"RIFF");
	assert_eq!(&data[8..12], b"WAVE");
	assert_eq!(&data[12..16], b"fmt ");
	assert_eq!(u16::from_le_bytes([data[20], data[21]]), 1);
	assert_eq!(u16::from_le_bytes([data[22], data[23]]), 2);
	assert_eq!(u32::from_le_bytes([data[24], data[25], data[26], data[27]]), 44_100);
	assert_eq!(u16::from_le_bytes([data[34], data[35]]), 16);
	assert_eq!(&data[36..40], b"data");
	assert_eq!(u32::from_le_bytes([data[40], data[41], data[42], data[43]]), 4);
	assert_eq!(&data[44..48], &[1, 2, 3, 4]);
}

#[test]
fn wav_muxer_rejects_second_stream() {
	let io = SharedIo::default();
	let mut muxer = WavMuxer::new(Box::new(io));

	let stream = Stream::audio(StreamId(0), 44_100, 2, CodecId::new("pcm_s16le"));
	muxer.add(&stream).unwrap();
	assert!(muxer.add(&stream).is_err());
}

#[test]
fn wav_muxer_uses_float_format_tag_when_codec_is_float() {
	let io = SharedIo::default();
	let mut muxer = WavMuxer::new(Box::new(io.clone()));

	let stream = Stream::audio(StreamId(0), 48_000, 1, CodecId::new("pcm_f32le"));
	muxer.add(&stream).unwrap();
	muxer.finish().unwrap();

	let data = io.snapshot();
	assert_eq!(u16::from_le_bytes([data[20], data[21]]), 3);
	assert_eq!(u16::from_le_bytes([data[34], data[35]]), 32);
}

#[derive(Clone, Default)]
struct SharedIo {
	inner: std::sync::Arc<std::sync::Mutex<BufferIo>>,
}

impl SharedIo {
	fn snapshot(&self) -> Vec<u8> {
		self.inner.lock().unwrap().data.clone()
	}
}

impl Io for SharedIo {
	fn read(&mut self, buf: &mut [u8]) -> ffmpreg::Result<usize> {
		self.inner.lock().unwrap().read(buf)
	}

	fn write(&mut self, buf: &[u8]) -> ffmpreg::Result<usize> {
		self.inner.lock().unwrap().write(buf)
	}

	fn write_all(&mut self, buf: &[u8]) -> ffmpreg::Result<()> {
		self.inner.lock().unwrap().write_all(buf)
	}

	fn seek(&mut self, pos: u64) -> ffmpreg::Result<()> {
		self.inner.lock().unwrap().seek(pos)
	}

	fn size(&self) -> ffmpreg::Result<u64> {
		self.inner.lock().unwrap().size()
	}
}

#[derive(Default)]
struct BufferIo {
	data: Vec<u8>,
	cursor: usize,
}

impl Io for BufferIo {
	fn read(&mut self, buf: &mut [u8]) -> ffmpreg::Result<usize> {
		if self.cursor >= self.data.len() {
			return Ok(0);
		}
		let remaining = &self.data[self.cursor..];
		let n = remaining.len().min(buf.len());
		buf[..n].copy_from_slice(&remaining[..n]);
		self.cursor += n;
		Ok(n)
	}

	fn write(&mut self, buf: &[u8]) -> ffmpreg::Result<usize> {
		let end = self.cursor + buf.len();
		if end > self.data.len() {
			self.data.resize(end, 0);
		}
		self.data[self.cursor..end].copy_from_slice(buf);
		self.cursor = end;
		Ok(buf.len())
	}

	fn seek(&mut self, pos: u64) -> ffmpreg::Result<()> {
		self.cursor = pos as usize;
		Ok(())
	}

	fn size(&self) -> ffmpreg::Result<u64> {
		Ok(self.data.len() as u64)
	}
}
