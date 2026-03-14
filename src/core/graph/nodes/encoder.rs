use crate::core::graph::{Media, Node};
use crate::core::traits::Encoder;
use crate::message::Result;

pub struct EncoderNode {
	encoder: Box<dyn Encoder>,
}

impl EncoderNode {
	pub fn new(encoder: Box<dyn Encoder>) -> Self {
		Self { encoder }
	}
}

impl Node for EncoderNode {
	fn run(&mut self, input: Media) -> Result<Vec<Media>> {
		let frame = input.into_frame()?;
		let packets = self.encoder.encode(frame)?;
		let output = packets.map(Media::Packet).collect();
		Ok(output)
	}

	fn flush(&mut self) -> Result<Vec<Media>> {
		let packets = self.encoder.finish()?;
		let output = packets.map(Media::Packet).collect();
		Ok(output)
	}
}
