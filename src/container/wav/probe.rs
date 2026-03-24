use crate::core::{Demuxer, Hint, Muxer};
use crate::message::Result;
use crate::{core::Probe, io::Io};

/// RIFF (4 bytes)
const RIFF: &[u8] = b"RIFF";
/// file size (4 bytes)
const FILE_SIZE: usize = 4;
/// WAVE (4 bytes)
const WAVE: &[u8] = b"WAVE";

/// header size = RIFF (4 bytes) + file size (4 bytes) + WAVE (4 bytes)
const RIFF_HEADER_SIZE: usize = RIFF.len() + FILE_SIZE + WAVE.len();

pub struct WavProbe;

impl Probe for WavProbe {
	fn demuxer_matches(data: &[u8]) -> bool {
		if data.len() < RIFF_HEADER_SIZE {
			return false;
		}

		&data[0..4] == RIFF && &data[8..12] == WAVE
	}

	fn muxer_matches(hint: &Hint) -> bool {
		if let Some(mime) = hint.mime_type.as_ref().map(|x| x.trim()) {
			if mime == "audio/wav" || mime == "audio/x-wav" {
				return true;
			}
		}

		if hint.extension.as_ref().is_some_and(|e| e == "wav") {
			return true;
		}
		false
	}

	fn create_muxer(io: Box<dyn Io>) -> Result<Box<dyn Muxer>> {
		Ok(Box::new(super::muxer::WavMuxer::new(io)))
	}

	fn create_demuxer(io: Box<dyn Io>) -> Result<Box<dyn Demuxer>> {
		Ok(Box::new(super::demuxer::WavDemuxer::new(io)?))
	}
}
