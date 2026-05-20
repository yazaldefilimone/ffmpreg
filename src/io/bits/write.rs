use std::io::{self, Write};

const BYTE_BITS: u8 = 8;
const MAX_BUFFER_BITS: u8 = 64;

pub struct Writer<W> {
	writer: W,
	buffer: u64,
	bit_count: u8,
}

impl<W: Write> Writer<W> {
	pub fn new(writer: W) -> Self {
		Self { writer, buffer: 0, bit_count: 0 }
	}

	pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
		self.write_bits(u64::from(bit), 1)
	}

	pub fn write_bits(&mut self, value: u64, count: u8) -> io::Result<()> {
		self.validate_write(value, count)?;

		if count == 0 {
			return Ok(());
		}

		for shift in (0..count).rev() {
			self.push_bit(((value >> shift) & 1) != 0)?;
		}

		Ok(())
	}

	pub fn align_to_byte(&mut self, fill_bit: bool) -> io::Result<u8> {
		let remainder = self.bit_count % BYTE_BITS;
		if remainder == 0 {
			return Ok(0);
		}

		let padding = BYTE_BITS - remainder;
		for _ in 0..padding {
			self.push_bit(fill_bit)?;
		}

		Ok(padding)
	}

	pub fn flush(&mut self) -> io::Result<()> {
		self.align_to_byte(false)?;
		self.writer.flush()
	}

	pub fn bits_buffered(&self) -> u8 {
		self.bit_count
	}

	pub fn into_inner(mut self) -> io::Result<W> {
		self.flush()?;
		Ok(self.writer)
	}

	fn push_bit(&mut self, bit: bool) -> io::Result<()> {
		self.buffer = (self.buffer << 1) | u64::from(bit);
		self.bit_count += 1;

		if self.bit_count >= BYTE_BITS {
			self.flush_full_bytes()?;
		}

		Ok(())
	}

	fn flush_full_bytes(&mut self) -> io::Result<()> {
		while self.bit_count >= BYTE_BITS {
			let shift = self.bit_count - BYTE_BITS;
			let byte = ((self.buffer >> shift) & 0xFF) as u8;
			self.writer.write_all(&[byte])?;
			self.bit_count -= BYTE_BITS;
			self.buffer &= low_bits_mask(self.bit_count);
		}

		Ok(())
	}

	fn validate_write(&self, value: u64, count: u8) -> io::Result<()> {
		let kind = io::ErrorKind::InvalidInput;

		if count > MAX_BUFFER_BITS {
			return Err(io::Error::new(kind, "bit count must be between 0 and 64"));
		}

		if count < MAX_BUFFER_BITS && value > low_bits_mask(count) {
			return Err(io::Error::new(kind, "value does not fit in the requested bit count"));
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
