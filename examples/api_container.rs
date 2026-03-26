use ffmpreg::Result;
use ffmpreg::core::traits::*;
use ffmpreg::io::{Input, Output};

const URL: &str = "https://samplefile.com/samples/download/audio/wav/wav_stereo_44k_mix_sample.wav";
const FILE: &str = "./playground/input.wav";

pub fn main() -> Result<()> {
	let mut demuxer = Input::open(FILE)?;
	let mut muxer = Output::create("./playground/output.wav")?;

	// muxer.add_all(demuxer.streams())?;
	for stream in demuxer.streams().iter() {
		let _metadata = &stream.metadata;
		muxer.add(stream)?;
	}

	let time = demuxer.duration();
	let metadata = demuxer.metadata();
	println!("metadata:\n{}", metadata);
	// let (_hour, _minutes, _seconds, _milliseconds) = time.unpack();

	println!("duration: {}", time);

	while let Some(packet) = demuxer.read()? {
		muxer.write(packet)?;
	}

	muxer.finish()
}
