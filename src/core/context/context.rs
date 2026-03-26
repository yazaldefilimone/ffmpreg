use crate::core::{Decoder, Encoder, State, StreamId};

pub struct Context {
	pub stream_id: StreamId,
	pub decoder: Box<dyn Decoder>,
	pub encoder: Box<dyn Encoder>,
	pub state_decode: State,
	pub state_encode: State,
}
