use super::{AspectRatio, Colorspace, Interlacing, Y4mFormat};
use crate::core::{Demuxer, Packet, Timebase};
use crate::io::{BufferedReader, IoError, IoResult, MediaRead, ReadPrimitives};

pub struct Y4mReader<R: MediaRead> {
	reader: BufferedReader<R>,
	format: Y4mFormat,
	timebase: Timebase,
	frame_count: u64,
}

impl<R: MediaRead> Y4mReader<R> {
	pub fn new(reader: R) -> IoResult<Self> {
		let mut buf_reader = BufferedReader::new(reader);
		let format = Self::read_header(&mut buf_reader)?;
		let timebase = Timebase::new(format.framerate_den, format.framerate_num);

		Ok(Self { reader: buf_reader, format, timebase, frame_count: 0 })
	}

	pub fn format(&self) -> Y4mFormat {
		self.format.clone()
	}

	fn read_header(reader: &mut BufferedReader<R>) -> IoResult<Y4mFormat> {
		let mut header = Vec::new();
		loop {
			let byte = reader.read_u8()?;
			if byte == b'\n' {
				break;
			}
			header.push(byte);
		}

		let header_str = core::str::from_utf8(&header)
			.map_err(|_| IoError::invalid_data("invalid UTF-8 in header"))?;

		if !header_str.starts_with("YUV4MPEG2") {
			return Err(IoError::invalid_data("not a Y4M file"));
		}

		let mut format = Y4mFormat::default();

		for param in header_str.split_whitespace().skip(1) {
			if param.is_empty() {
				continue;
			}
			let (key, value) = param.split_at(1);
			match key {
				"W" => format.width = value.parse().unwrap_or(format.width),
				"H" => format.height = value.parse().unwrap_or(format.height),
				"F" => {
					if let Some((num, den)) = value.split_once(':') {
						format.framerate_num = num.parse().unwrap_or(30);
						format.framerate_den = den.parse().unwrap_or(1);
					}
				}
				"I" => {
					if let Some(c) = value.chars().next() {
						format.interlacing = Interlacing::from_char(c).unwrap_or(Interlacing::Progressive);
					}
				}
				"C" => {
					format.colorspace = Colorspace::from_str(value);
				}
				"A" => {
					format.aspect_ratio = AspectRatio::from_str(value);
				}
				_ => {}
			}
		}

		Ok(format)
	}

	fn read_frame_header(&mut self) -> IoResult<bool> {
		let mut header = Vec::new();
		loop {
			match self.reader.read_u8() {
				Ok(byte) => {
					if byte == b'\n' {
						break;
					}
					header.push(byte);
				}
				Err(e) if matches!(e.kind(), crate::io::IoErrorKind::UnexpectedEof) => {
					if header.is_empty() {
						return Ok(false);
					}
					return Err(e);
				}
				Err(e) => return Err(e),
			}
		}

		let header_str = core::str::from_utf8(&header)
			.map_err(|_| IoError::invalid_data("invalid UTF-8 in frame header"))?;

		if !header_str.starts_with("FRAME") {
			return Err(IoError::invalid_data("expected FRAME header"));
		}

		Ok(true)
	}
}

impl<R: MediaRead> Demuxer for Y4mReader<R> {
	fn read_packet(&mut self) -> IoResult<Option<Packet>> {
		if !self.read_frame_header()? {
			return Ok(None);
		}

		let frame_size = self.format.frame_size();
		let mut data = vec![0u8; frame_size];
		self.reader.read_exact(&mut data)?;

		let pts = self.frame_count as i64;
		self.frame_count += 1;

		Ok(Some(Packet::new(data, 0, self.timebase).with_pts(pts)))
	}

	fn stream_count(&self) -> usize {
		1
	}
}
