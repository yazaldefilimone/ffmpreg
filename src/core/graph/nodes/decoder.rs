use crate::core::graph::{Media, Node};
use crate::core::traits::Decoder;
use crate::message::Result;

pub struct DecoderNode {
	decoder: Box<dyn Decoder>,
}

impl DecoderNode {
	pub fn new(decoder: Box<dyn Decoder>) -> Self {
		Self { decoder }
	}
}

impl Node for DecoderNode {
	fn run(&mut self, input: Media) -> Result<Vec<Media>> {
		let packet = input.into_packet()?;
		let frames = self.decoder.decode(packet)?;
		let output = frames.map(Media::Frame).collect();
		Ok(output)
	}

	fn flush(&mut self) -> Result<Vec<Media>> {
		let frames = self.decoder.finish()?;
		let output = frames.map(Media::Frame).collect();
		Ok(output)
	}
}
