use crate::Result;
use crate::core::*;

pub struct ContextState {
	pub decode: State,
	pub encode: State,
}

impl ContextState {
	pub fn new(decode_state: State, encode_state: State) -> Self {
		Self { decode: decode_state, encode: encode_state }
	}
}

pub struct Context {
	pub stream_id: StreamId,
	pub decoder: Box<dyn Decoder>,
	pub encoder: Box<dyn Encoder>,
	pub state: ContextState,
}

impl Context {
	pub fn encode(&mut self, frame: Frame) -> Result<EncodeOut<State>> {
		self.encoder.encode(&mut self.state.encode, frame)
	}

	pub fn decode(&mut self, packet: Packet) -> Result<DecodeOut<State>> {
		self.decoder.decode(&mut self.state.decode, packet)
	}

	pub fn run(&mut self, packet: Packet) -> Result<Vec<Packet>> {
		let mut packets = Vec::new();
		let output = self.decode(packet)?;
		for frame in output.value {
			let output = self.encode(frame)?;
			packets.extend(output.value);
		}
		Ok(packets)
	}

	// pub fn decode_flush(&mut self) -> Result<DecodeOut<State>> {
	// 	todo!()
	// }

	pub fn flush(&mut self) -> Result<EncodeOut<State>> {
		self.encoder.flush(&mut self.state.encode)
	}

	// pub fn decode_flush(&mut self) -> Result<DecodeOut<State>> {
	// 	todo!()
	// }
}
