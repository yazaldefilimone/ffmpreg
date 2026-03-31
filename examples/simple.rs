use ffmpreg::Result;
use ffmpreg::core::Resolver;
use ffmpreg::core::StreamHashMap;
use ffmpreg::core::traits::*;
use ffmpreg::core::{Context, ContextState};
use ffmpreg::io::{Input, Output};

pub fn main() -> Result<()> {
	let mut demuxer = Input::open("./playground/your_name_sparkle.wav")?;
	let mut muxer = Output::create("./playground/output.wav")?;

	let resolver = Resolver::new();
	let mut streams = StreamHashMap::default();

	for stream in demuxer.streams().iter() {
		let decoder = resolver.decoder_for(stream).unwrap();
		let encoder = resolver.encoder_for(stream).unwrap();

		let decode_state = decoder.create_state(&stream.parameters);
		let encode_state = encoder.create_state(&stream.parameters);
		let state = ContextState::new(decode_state, encode_state);
		let context = Context { stream_id: stream.id, decoder, encoder, state };
		streams.insert(stream.id, context);
		muxer.add(&stream)?;
	}

	while let Some(packet) = demuxer.read()? {
		let ctx = streams.get_mut(&packet.stream_id).unwrap();
		for packet in ctx.run(packet)? {
			muxer.write(packet)?;
		}
	}

	for ctx in streams.values_mut() {
		let output = ctx.flush()?;
		for packet in output.value {
			muxer.write(packet)?;
		}
	}

	muxer.finish()
}
