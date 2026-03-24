use crate::{
	core::{Frame, Hint, Packet, Parameters, State, Stream, StreamSet, Time},
	io::Io,
	message::Result,
};

pub struct StepOutput<O, S> {
	pub value: O,
	pub state: S,
}

pub type DecodeOut<S> = StepOutput<Vec<Frame>, S>;
pub type EncodeOut<S> = StepOutput<Vec<Packet>, S>;
pub type FilterOut<S> = StepOutput<Vec<Frame>, S>;

pub trait Decoder {
	fn create_state(&self, parameters: &Parameters) -> State;
	fn decode(&self, state: &mut State, packet: Packet) -> Result<DecodeOut<State>>;
	fn flush(&self, state: &mut State) -> Result<DecodeOut<State>>;
}

pub trait Encoder {
	fn create_state(&self, parameters: &Parameters) -> State;
	fn encode(&self, state: &mut State, frame: Frame) -> Result<EncodeOut<State>>;
	fn flush(&self, state: &mut State) -> Result<EncodeOut<State>>;
}

pub trait Filter {
	type State;
	fn filter(&self, state: &mut Self::State, frame: Frame) -> Result<FilterOut<Self::State>>;
	fn flush(&self, state: &mut Self::State) -> Result<FilterOut<Self::State>>;
}

pub trait Demuxer {
	fn read(&mut self) -> Result<Option<Packet>>;
	fn seek(&mut self, time: f64) -> Result<()>;
	fn duration(&self) -> Time;
	fn streams(&self) -> &StreamSet;
}

pub trait Muxer {
	fn add(&mut self, stream: &Stream) -> Result<usize>;

	fn add_all(&mut self, setter: &StreamSet) -> Result<usize> {
		for stream in setter.streams.iter() {
			self.add(stream)?;
		}
		Ok(0)
	}

	fn write(&mut self, packet: Packet) -> Result<()>;
	fn finish(&mut self) -> Result<()>;
}

pub trait Probe {
	fn demuxer_matches(data: &[u8]) -> bool;
	fn muxer_matches(hint: &Hint) -> bool;
	fn create_demuxer(io: Box<dyn Io>) -> Result<Box<dyn Demuxer>>;
	fn create_muxer(io: Box<dyn Io>) -> Result<Box<dyn Muxer>>;
}
