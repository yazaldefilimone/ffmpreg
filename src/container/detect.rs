use crate::Message;
use crate::container::wav;
use crate::container::y4m;
use crate::core::Demuxer;
use crate::core::Probe;
use crate::io::Io;
use crate::message::Result;

pub fn detect(data: &[u8], io: Box<dyn Io>) -> Result<Box<dyn Demuxer>> {
	if wav::WavProbe::demuxer_matches(data) {
		return wav::WavProbe::create_demuxer(io);
	}

	if y4m::Y4mProbe::demuxer_matches(data) {
		return y4m::Y4mProbe::create_demuxer(io);
	}

	Err(Message::Container("unsupported container format"))
}
