use crate::Message;
use crate::container::{riff, y4m};
use crate::core::{Hint, Muxer, Probe};
use crate::io::Io;
use crate::message::Result;

pub fn select_muxer(hint: Hint, io: Box<dyn Io>) -> Result<Box<dyn Muxer>> {
	if riff::RiffProbe::muxer_matches(&hint) {
		return riff::RiffProbe::create_muxer(io);
	}

	if y4m::Y4mProbe::muxer_matches(&hint) {
		return y4m::Y4mProbe::create_muxer(io);
	}

	Err(Message::Container("unsupported container format"))
}
