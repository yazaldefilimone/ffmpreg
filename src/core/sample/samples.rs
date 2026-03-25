pub enum SampleFormat {
	S16LE,
	F32LE,
}

pub enum Layout {
	Interleaved,
	Planar,
}

pub struct Samples {
	pub data: Vec<u8>,
	pub format: SampleFormat,
	pub channels: u16,
	pub sample_rate: u32,
	pub layout: Layout,
}
