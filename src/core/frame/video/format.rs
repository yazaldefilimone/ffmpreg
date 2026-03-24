#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
	YUV420,
	YUV422,
	YUV444,
}

#[derive(Debug, Clone)]
pub struct PixelFormat {
	pub depth: u8,
	pub format: Format,
}
