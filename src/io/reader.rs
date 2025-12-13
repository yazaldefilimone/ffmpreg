use crate::io::{IoError, IoResult};

pub const DEFAULT_BUFFER_SIZE: usize = 8192;

pub trait MediaRead {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>;
}

pub trait ReadPrimitives: MediaRead {
	fn read_exact(&mut self, buf: &mut [u8]) -> IoResult<()> {
		let mut filled = 0;
		while filled < buf.len() {
			match self.read(&mut buf[filled..]) {
				Ok(0) => return Err(IoError::unexpected_eof()),
				Ok(n) => filled += n,
				Err(e) if matches!(e.kind(), crate::io::IoErrorKind::Interrupted) => continue,
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	#[inline]
	fn read_u8(&mut self) -> IoResult<u8> {
		let mut buf = [0u8; 1];
		self.read_exact(&mut buf)?;
		Ok(buf[0])
	}

	#[inline]
	fn read_u16_be(&mut self) -> IoResult<u16> {
		let mut buf = [0u8; 2];
		self.read_exact(&mut buf)?;
		Ok(u16::from_be_bytes(buf))
	}

	#[inline]
	fn read_u16_le(&mut self) -> IoResult<u16> {
		let mut buf = [0u8; 2];
		self.read_exact(&mut buf)?;
		Ok(u16::from_le_bytes(buf))
	}

	#[inline]
	fn read_u32_be(&mut self) -> IoResult<u32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(u32::from_be_bytes(buf))
	}

	#[inline]
	fn read_u32_le(&mut self) -> IoResult<u32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(u32::from_le_bytes(buf))
	}

	#[inline]
	fn read_u64_be(&mut self) -> IoResult<u64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(u64::from_be_bytes(buf))
	}

	#[inline]
	fn read_u64_le(&mut self) -> IoResult<u64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(u64::from_le_bytes(buf))
	}

	#[inline]
	fn read_i8(&mut self) -> IoResult<i8> {
		let mut buf = [0u8; 1];
		self.read_exact(&mut buf)?;
		Ok(buf[0] as i8)
	}

	#[inline]
	fn read_i16_be(&mut self) -> IoResult<i16> {
		let mut buf = [0u8; 2];
		self.read_exact(&mut buf)?;
		Ok(i16::from_be_bytes(buf))
	}

	#[inline]
	fn read_i16_le(&mut self) -> IoResult<i16> {
		let mut buf = [0u8; 2];
		self.read_exact(&mut buf)?;
		Ok(i16::from_le_bytes(buf))
	}

	#[inline]
	fn read_i32_be(&mut self) -> IoResult<i32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(i32::from_be_bytes(buf))
	}

	#[inline]
	fn read_i32_le(&mut self) -> IoResult<i32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(i32::from_le_bytes(buf))
	}

	#[inline]
	fn read_i64_be(&mut self) -> IoResult<i64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(i64::from_be_bytes(buf))
	}

	#[inline]
	fn read_i64_le(&mut self) -> IoResult<i64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(i64::from_le_bytes(buf))
	}

	#[inline]
	fn read_f32_be(&mut self) -> IoResult<f32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(f32::from_be_bytes(buf))
	}

	#[inline]
	fn read_f32_le(&mut self) -> IoResult<f32> {
		let mut buf = [0u8; 4];
		self.read_exact(&mut buf)?;
		Ok(f32::from_le_bytes(buf))
	}

	#[inline]
	fn read_f64_be(&mut self) -> IoResult<f64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(f64::from_be_bytes(buf))
	}

	#[inline]
	fn read_f64_le(&mut self) -> IoResult<f64> {
		let mut buf = [0u8; 8];
		self.read_exact(&mut buf)?;
		Ok(f64::from_le_bytes(buf))
	}
}

impl<T: MediaRead> ReadPrimitives for T {}

pub struct StdReadAdapter<R> {
	inner: R,
}

impl<R> StdReadAdapter<R> {
	#[inline]
	pub const fn new(inner: R) -> Self {
		Self { inner }
	}

	#[inline]
	pub fn into_inner(self) -> R {
		self.inner
	}

	#[inline]
	pub const fn get_ref(&self) -> &R {
		&self.inner
	}

	#[inline]
	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}
}

impl<R: std::io::Read> MediaRead for StdReadAdapter<R> {
	#[inline]
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		self.inner.read(buf).map_err(IoError::from)
	}
}

pub struct BufferedReader<R, const N: usize = DEFAULT_BUFFER_SIZE> {
	inner: R,
	buffer: [u8; N],
	pos: usize,
	filled: usize,
}

impl<R, const N: usize> BufferedReader<R, N> {
	pub fn new(inner: R) -> Self {
		Self { inner, buffer: [0u8; N], pos: 0, filled: 0 }
	}

	#[inline]
	pub fn into_inner(self) -> R {
		self.inner
	}

	#[inline]
	pub const fn get_ref(&self) -> &R {
		&self.inner
	}

	#[inline]
	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}

	#[inline]
	pub fn buffer(&self) -> &[u8] {
		&self.buffer[self.pos..self.filled]
	}

	#[inline]
	pub const fn capacity(&self) -> usize {
		N
	}

	#[inline]
	fn consume(&mut self, amt: usize) {
		self.pos = core::cmp::min(self.pos + amt, self.filled);
	}

	#[inline]
	fn discard_buffer(&mut self) {
		self.pos = 0;
		self.filled = 0;
	}
}

impl<R: MediaRead, const N: usize> BufferedReader<R, N> {
	fn fill_buf(&mut self) -> IoResult<&[u8]> {
		if self.pos >= self.filled {
			self.discard_buffer();
			self.filled = self.inner.read(&mut self.buffer)?;
		}
		Ok(&self.buffer[self.pos..self.filled])
	}
}

impl<R: MediaRead, const N: usize> MediaRead for BufferedReader<R, N> {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		if buf.len() >= N && self.pos >= self.filled {
			self.discard_buffer();
			return self.inner.read(buf);
		}

		let available = self.fill_buf()?;
		let amt = core::cmp::min(available.len(), buf.len());
		buf[..amt].copy_from_slice(&available[..amt]);
		self.consume(amt);
		Ok(amt)
	}
}

impl MediaRead for &[u8] {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		let amt = core::cmp::min(self.len(), buf.len());
		let (a, b) = self.split_at(amt);
		buf[..amt].copy_from_slice(a);
		*self = b;
		Ok(amt)
	}
}
