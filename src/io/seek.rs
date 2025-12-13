use crate::io::IoResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
	Start(u64),
	End(i64),
	Current(i64),
}

impl From<SeekFrom> for std::io::SeekFrom {
	#[inline]
	fn from(pos: SeekFrom) -> Self {
		match pos {
			SeekFrom::Start(n) => std::io::SeekFrom::Start(n),
			SeekFrom::End(n) => std::io::SeekFrom::End(n),
			SeekFrom::Current(n) => std::io::SeekFrom::Current(n),
		}
	}
}

impl From<std::io::SeekFrom> for SeekFrom {
	#[inline]
	fn from(pos: std::io::SeekFrom) -> Self {
		match pos {
			std::io::SeekFrom::Start(n) => SeekFrom::Start(n),
			std::io::SeekFrom::End(n) => SeekFrom::End(n),
			std::io::SeekFrom::Current(n) => SeekFrom::Current(n),
		}
	}
}

pub trait MediaSeek {
	fn seek(&mut self, pos: SeekFrom) -> IoResult<u64>;

	#[inline]
	fn stream_position(&mut self) -> IoResult<u64> {
		self.seek(SeekFrom::Current(0))
	}

	#[inline]
	fn rewind(&mut self) -> IoResult<()> {
		self.seek(SeekFrom::Start(0))?;
		Ok(())
	}

	fn stream_len(&mut self) -> IoResult<u64> {
		let current = self.stream_position()?;
		let end = self.seek(SeekFrom::End(0))?;
		if current != end {
			self.seek(SeekFrom::Start(current))?;
		}
		Ok(end)
	}
}

pub struct StdSeekAdapter<S> {
	inner: S,
}

impl<S> StdSeekAdapter<S> {
	#[inline]
	pub const fn new(inner: S) -> Self {
		Self { inner }
	}

	#[inline]
	pub fn into_inner(self) -> S {
		self.inner
	}

	#[inline]
	pub const fn get_ref(&self) -> &S {
		&self.inner
	}

	#[inline]
	pub fn get_mut(&mut self) -> &mut S {
		&mut self.inner
	}
}

impl<S: std::io::Seek> MediaSeek for StdSeekAdapter<S> {
	#[inline]
	fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
		self.inner.seek(pos.into()).map_err(crate::io::IoError::from)
	}
}

pub struct SeekableReader<R, S> {
	reader: R,
	seeker: S,
}

impl<R, S> SeekableReader<R, S> {
	#[inline]
	pub const fn new(reader: R, seeker: S) -> Self {
		Self { reader, seeker }
	}

	#[inline]
	pub fn into_parts(self) -> (R, S) {
		(self.reader, self.seeker)
	}

	#[inline]
	pub const fn reader(&self) -> &R {
		&self.reader
	}

	#[inline]
	pub fn reader_mut(&mut self) -> &mut R {
		&mut self.reader
	}

	#[inline]
	pub const fn seeker(&self) -> &S {
		&self.seeker
	}

	#[inline]
	pub fn seeker_mut(&mut self) -> &mut S {
		&mut self.seeker
	}
}

impl<R: crate::io::MediaRead, S> crate::io::MediaRead for SeekableReader<R, S> {
	#[inline]
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		self.reader.read(buf)
	}
}

impl<R, S: MediaSeek> MediaSeek for SeekableReader<R, S> {
	#[inline]
	fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
		self.seeker.seek(pos)
	}
}

pub struct SeekableWriter<W, S> {
	writer: W,
	seeker: S,
}

impl<W, S> SeekableWriter<W, S> {
	#[inline]
	pub const fn new(writer: W, seeker: S) -> Self {
		Self { writer, seeker }
	}

	#[inline]
	pub fn into_parts(self) -> (W, S) {
		(self.writer, self.seeker)
	}

	#[inline]
	pub const fn writer(&self) -> &W {
		&self.writer
	}

	#[inline]
	pub fn writer_mut(&mut self) -> &mut W {
		&mut self.writer
	}

	#[inline]
	pub const fn seeker(&self) -> &S {
		&self.seeker
	}

	#[inline]
	pub fn seeker_mut(&mut self) -> &mut S {
		&mut self.seeker
	}
}

impl<W: crate::io::MediaWrite, S> crate::io::MediaWrite for SeekableWriter<W, S> {
	#[inline]
	fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
		self.writer.write(buf)
	}

	#[inline]
	fn flush(&mut self) -> IoResult<()> {
		self.writer.flush()
	}
}

impl<W, S: MediaSeek> MediaSeek for SeekableWriter<W, S> {
	#[inline]
	fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
		self.seeker.seek(pos)
	}
}
