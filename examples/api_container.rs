use ffmpreg::Result;
use ffmpreg::core::traits::*;
use ffmpreg::io::{Input, Output};

const FILE: &str = "./playground/output.wav";

pub fn main() -> Result<()> {
	let mut demuxer = Input::open(FILE)?;
	let mut muxer = Output::create("./playground/output.wav")?;

	muxer.add_all(demuxer.streams())?;

	while let Some(packet) = demuxer.read()? {
		muxer.write(packet)?;
	}

	muxer.finish()
}
