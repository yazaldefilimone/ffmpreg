use osaka::Result;
use osaka::core::resolver::*;
use osaka::io::{Input, Output};

fn main() -> Result<()> {
	let mut input = Input::open("./playground/sparkle.flac")?;
	let mut output = Output::new("./output_decoded.wav")?;

	let mut audio_track = input.tracks.primary_audio().copied()?;

	let resolver = CodecResolver::new();

	let codec_out = resolver.codec_for_extension(&output.extension)?;

	audio_track.codec_out(codec_out);

	let mut decoder = resolver.decoder_for(&audio_track)?;
	let mut encoder = resolver.encoder_for(&audio_track, output.format())?;

	let meta = input.metadata();
	if let Some(artist) = meta.artist.as_ref() {
		println!("artist: {}", artist)
	}

	for packet in input.iter_packets() {
		for frame in decoder.decode(packet?)? {
			for packet in encoder.encode(frame)? {
				output.write_packet(packet)?;
			}
		}
	}

	for frame in decoder.finish()? {
		output.write_all(encoder.encode(frame)?)?;
	}

	output.write_all(encoder.finish()?)?;

	output.flush()
}
