use crate::core::{Demuxer, Hint, Muxer};
use crate::message::Result;
use crate::{core::Probe, io::Io};

const Y4M_MAGIC: &[u8] = b"YUV4MPEG2";

pub struct Y4mProbe;

impl Probe for Y4mProbe {
	fn demuxer_matches(data: &[u8]) -> bool {
		if data.len() < Y4M_MAGIC.len() {
			return false;
		}
		&data[0..Y4M_MAGIC.len()] == Y4M_MAGIC
	}

	fn muxer_matches(hint: &Hint) -> bool {
		if let Some(mime) = hint.mime_type.as_ref().map(|x| x.trim()) {
			if mime == "video/x-yuv4mpegpipe" {
				return true;
			}
		}

		if hint.extension.as_ref().is_some_and(|e| e == "y4m") {
			return true;
		}
		false
	}

	fn create_demuxer(_io: Box<dyn Io>) -> Result<Box<dyn Demuxer>> {
		todo!()
	}

	fn create_muxer(_io: Box<dyn Io>) -> Result<Box<dyn Muxer>> {
		todo!()
	}
}
