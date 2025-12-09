pub mod read;
pub mod write;

pub use read::WavReader;
pub use write::WavWriter;

#[derive(Debug, Clone, Copy)]
pub struct WavFormat {
	pub channels: u8,
	pub sample_rate: u32,
	pub bit_depth: u16,
}

impl WavFormat {
	pub fn bytes_per_sample(&self) -> usize {
		(self.bit_depth / 8) as usize
	}

	pub fn bytes_per_frame(&self) -> usize {
		self.bytes_per_sample() * self.channels as usize
	}
}
