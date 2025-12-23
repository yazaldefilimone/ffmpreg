use super::time::Timebase;

#[derive(Debug, Clone)]
pub struct Packet {
	pub data: Vec<u8>,
	pub pts: i64,
	pub dts: i64,
	pub timebase: Timebase,
	pub stream_index: usize,
	pub keyframe: bool,
	pub discard: bool,
}

impl Packet {
	pub fn new(data: Vec<u8>, stream_index: usize, timebase: Timebase) -> Self {
		Self { data, pts: 0, dts: 0, timebase, stream_index, keyframe: false, discard: false }
	}

	pub fn with_pts(mut self, pts: i64) -> Self {
		self.pts = pts;
		self
	}

	pub fn with_dts(mut self, dts: i64) -> Self {
		self.dts = dts;
		self
	}

	pub fn size(&self) -> usize {
		self.data.len()
	}

	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}
