use osaka::Result;
use osaka::core::resolver::CodecResolver;
use osaka::io::Input;
use osaka::message::Report;
use std::time::Instant;

fn main() -> Result<()> {
	let start = Instant::now();
	let mut input = Input::open("./playground/sparkle.wav")?;
	let resolver = CodecResolver::new();

	let audio = input.tracks.primary_audio()?;

	let mut decoder = resolver.decoder_for(&audio)?;

	let mut frames = 0;
	for packet in input.iter_packets() {
		let packet = packet.report();
		for frame in decoder.decode(packet)? {
			frames += 1;
		}
	}

	let duration = start.elapsed().as_secs_f64();
	let fps = frames as f64 / duration;
	Ok(())
}
