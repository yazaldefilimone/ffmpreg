#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
	YUV420,
	YUV422,
	YUV444,
}

#[derive(Debug, Clone)]
pub struct Pixel {
	pub depth: u8,
	pub format: PixelFormat,
}
