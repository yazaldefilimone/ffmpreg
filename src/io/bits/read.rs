use std::io::{self, Read};

const BYTE_BITS: u8 = 8;
const MAX_BUFFER_BITS: u8 = 64;

pub struct Reader<R> {
	reader: R,
	buffer: u64,
	bit_count: u8,
}

impl<R: Read> Reader<R> {
	pub fn new(reader: R) -> Self {
		Self { reader, buffer: 0, bit_count: 0 }
	}

	pub fn read_bit(&mut self) -> io::Result<bool> {
		Ok(self.read_bits(1)? != 0)
	}

	pub fn read_bits(&mut self, count: u8) -> io::Result<u64> {
		let value = self.peek_bits(count)?;
		self.skip_bits(count)?;
		Ok(value)
	}

	pub fn peek_bits(&mut self, count: u8) -> io::Result<u64> {
		self.check_bit_count(count)?;
		self.fill_to(count)?;

		if count == 0 {
			return Ok(0);
		}

		let shift = self.bit_count - count;
		Ok((self.buffer >> shift) & low_bits_mask(count))
	}

	pub fn skip_bits(&mut self, count: u8) -> io::Result<()> {
		self.check_bit_count(count)?;

		if count > self.bit_count {
			let kind = io::ErrorKind::UnexpectedEof;
			return Err(io::Error::new(kind, "cannot skip more bits than currently buffered"));
		}

		self.bit_count -= count;
		self.buffer &= low_bits_mask(self.bit_count);
		Ok(())
	}

	pub fn align_to_byte(&mut self) -> io::Result<u8> {
		let remainder = self.bit_count % BYTE_BITS;
		if remainder == 0 {
			return Ok(0);
		}

		self.skip_bits(remainder)?;
		Ok(remainder)
	}

	pub fn bits_buffered(&self) -> u8 {
		self.bit_count
	}

	pub fn into_inner(self) -> R {
		self.reader
	}

	fn fill_to(&mut self, count: u8) -> io::Result<()> {
		while self.bit_count < count {
			if self.bit_count > MAX_BUFFER_BITS - BYTE_BITS {
				let kind = io::ErrorKind::InvalidInput;
				return Err(io::Error::new(kind, "bit request exceeds internal buffer capacity"));
			}

			let mut byte = [0u8; 1];
			self.reader.read_exact(&mut byte)?;
			self.buffer = (self.buffer << BYTE_BITS) | u64::from(byte[0]);
			self.bit_count += BYTE_BITS;
		}

		Ok(())
	}

	fn check_bit_count(&self, count: u8) -> io::Result<()> {
		if count > MAX_BUFFER_BITS {
			let kind = io::ErrorKind::InvalidInput;
			return Err(io::Error::new(kind, "bit count must be between 0 and 64"));
		}

		Ok(())
	}
}

fn low_bits_mask(bits: u8) -> u64 {
	match bits {
		0 => 0,
		64 => u64::MAX,
		_ => (1u64 << bits) - 1,
	}
}
