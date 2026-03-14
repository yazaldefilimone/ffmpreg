use crate::{core::time::Timestamp, message};

pub mod audio;
pub mod iter;
pub mod subtitle;
pub mod video;
pub use audio::*;
pub use iter::*;
pub use subtitle::*;
pub use video::*;
#[derive(Debug, Clone)]

pub enum MediaFrame {
	Audio(AudioFrame),
	Video(VideoFrame),
}

#[derive(Debug, Clone)]
pub struct Frame {
	pub stream_id: usize,
	pub data: MediaFrame,
	pub pts: Timestamp,
}

impl Frame {
	pub fn map_audio<F>(&mut self, func: F) -> message::Result<()>
	where
		F: FnOnce(AudioFrame) -> message::Result<AudioFrame>,
	{
		if let MediaFrame::Audio(inner) =
			std::mem::replace(&mut self.data, MediaFrame::Audio(AudioFrame::empty()))
		{
			self.data = MediaFrame::Audio(func(inner)?);
		}
		Ok(())
	}

	pub fn map_video<F>(&mut self, func: F) -> message::Result<()>
	where
		F: FnOnce(VideoFrame) -> message::Result<VideoFrame>,
	{
		if let MediaFrame::Video(inner) =
			std::mem::replace(&mut self.data, MediaFrame::Video(VideoFrame::empty()))
		{
			self.data = MediaFrame::Video(func(inner)?);
		}
		Ok(())
	}

	pub fn is_audio(&self) -> bool {
		matches!(self.data, MediaFrame::Audio(_))
	}

	pub fn is_video(&self) -> bool {
		matches!(self.data, MediaFrame::Video(_))
	}

	pub fn pts(&self) -> Timestamp {
		self.pts
	}
}
