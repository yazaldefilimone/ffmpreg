use crate::core::{Packet, StreamSet, Time};
use crate::{container, message::*};
use crate::{core::Demuxer, io::*};

pub struct Input {
	demuxer: Box<dyn Demuxer>,
}

impl Input {
	pub fn open(input: impl AsRef<str>) -> Result<Self> {
		match source::parse_source(input.as_ref())? {
			source::Source::File(path) => {
				let file = File::open(path)?;
				Self::from_io(Box::new(file))
			}
			source::Source::Url(url) => {
				let http_io = HttpIo::open(&url)?;
				Self::from_io(Box::new(http_io))
			}
		}
	}

	fn from_io(io: Box<dyn Io>) -> Result<Self> {
		let mut io = io;
		let mut buffer = vec![0u8; 4096];
		let size = io.read(&mut buffer)?;
		buffer.truncate(size);

		io.seek(0)?;

		let demuxer = container::detect(&buffer[..], io)?;
		Ok(Self { demuxer })
	}
}

impl Demuxer for Input {
	fn read(&mut self) -> Result<Option<Packet>> {
		self.demuxer.read()
	}

	fn streams(&self) -> &StreamSet {
		self.demuxer.streams()
	}

	fn seek(&mut self, time: f64) -> Result<()> {
		self.demuxer.seek(time)
	}

	fn duration(&self) -> Time {
		self.demuxer.duration()
	}
}
