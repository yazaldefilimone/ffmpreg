use ffmpreg::Result;
use ffmpreg::core::Context;
use ffmpreg::core::Resolver;
use ffmpreg::core::StreamHashMap;
use ffmpreg::core::traits::*;
use ffmpreg::io::{Input, Output};

pub fn main() -> Result<()> {
	let mut demuxer = Input::open("./playground/your_name_sparkle.wav")?;
	let mut muxer = Output::create("./playground/output.wav")?;

	let resolver = Resolver::new();
	let mut streams = StreamHashMap::default();

	for stream in demuxer.streams().iter() {
		let decoder = resolver.decoder_for(stream).unwrap();
		let encoder = resolver.encoder_for(stream).unwrap();

		let state_decode = decoder.create_state(&stream.parameters);
		let state_encode = encoder.create_state(&stream.parameters);
		let context = Context { stream_id: stream.id, decoder, encoder, state_decode, state_encode };
		streams.insert(stream.id, context);
		muxer.add(&stream)?;
	}

	while let Some(packet) = demuxer.read()? {
		let ctx = streams.get_mut(&packet.stream_id).unwrap();

		let output = ctx.decoder.decode(&mut ctx.state_decode, packet)?;
		for frame in output.value {
			let output = ctx.encoder.encode(&mut ctx.state_encode, frame)?;
			for packet in output.value {
				muxer.write(packet)?;
			}
		}
	}

	for ctx in streams.values_mut() {
		let output = ctx.encoder.flush(&mut ctx.state_encode)?;
		for packet in output.value {
			muxer.write(packet)?;
		}
	}

	muxer.finish()
}
