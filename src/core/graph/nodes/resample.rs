use crate::core::graph::{Media, Node};
use crate::core::traits::Resampler;
use crate::message::Result;

pub struct ResamplerNode {
	resampler: Box<dyn Resampler>,
}

impl ResamplerNode {
	pub fn new(resampler: Box<dyn Resampler>) -> Self {
		Self { resampler }
	}
}

impl Node for ResamplerNode {
	fn run(&mut self, input: Media) -> Result<Vec<Media>> {
		let frame = input.into_frame()?;
		let output = self.resampler.resample(frame)?;
		Ok(vec![Media::Frame(output)])
	}

	fn flush(&mut self) -> Result<Vec<Media>> {
		Ok(Vec::new())
	}
}
