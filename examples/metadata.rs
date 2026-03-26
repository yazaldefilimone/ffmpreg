use ffmpreg::Result;
use ffmpreg::core::traits::{Demuxer, Muxer};
use ffmpreg::core::{Metadata, Stream};
use ffmpreg::io::{Input, Output};

pub fn main() -> Result<()> {
	let input = Input::open("./playground/your_name_sparkle.wav")?;

	let global = input.metadata();
	println!("global title: {:?}", global.title);

	if let Some(stream) = input.streams().iter().next() {
		println!("stream {:?} title: {:?}", stream.id, stream.metadata.title);
	}

	let mut output = Output::create("./playground/output.wav")?;
	output.set_metadata(Metadata {
		title: Some("global title".into()),
		artist: Some("global artist".into()),
		..Default::default()
	})?;

	let mut stream =
		Stream::audio(0usize.into(), 48_000, 2, ffmpreg::core::CodecId::new("pcm_s16le"));
	stream.metadata = Metadata {
		title: Some("stream title".into()),
		comment: Some("audio stream".into()),
		..Default::default()
	};

	output.add(&stream)?;
	Ok(())
}
