use super::Media;
use crate::message::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
	pub fn new(value: usize) -> Self {
		Self(value)
	}

	pub fn value(&self) -> usize {
		self.0
	}
}

pub trait Node {
	fn run(&mut self, input: Media) -> Result<Vec<Media>>;
	fn flush(&mut self) -> Result<Vec<Media>>;
}
