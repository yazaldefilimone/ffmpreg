use crate::Result;
use crate::core;
use crate::core::AudioState;
use crate::core::Channels;
use crate::core::CodecState;
use crate::core::DecodeOut;
use crate::core::SampleFormat;
use crate::core::SampleLayout;
use crate::core::State;

pub struct PcmDecoder {}

impl core::Decoder for PcmDecoder {
	fn create_state(&self, p: &core::Parameters) -> State {
		let base = CodecState { extradata: Vec::new(), pts: None, dts: None, last_pts: None };
		let audio_state = AudioState {
			sample_rate: p.sample_rate.unwrap_or(44100),
			channels: Channels::from_value(p.channels.unwrap_or(2)),
			format: SampleFormat::S16,
			layout: SampleLayout::Interleaved,
			frame_size: 0,
			base,
			samples_consumed: 0,
			samples_produced: 0,
			buffer: Vec::new(),
		};
		State::Audio(audio_state)
	}

	fn decode(&self, state: &mut State, packet: core::Packet) -> Result<DecodeOut<State>> {
		todo!()
	}

	fn flush(&self, state: &mut State) -> Result<DecodeOut<State>> {
		todo!()
	}
}
