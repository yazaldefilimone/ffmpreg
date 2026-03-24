pub mod audio;
pub mod video;

pub use audio::{AudioFrame, AudioParams, ChannelLayout, SampleFormat, SampleRate};
pub use video::{Keyframe, PixelFormat, PixelFormatKind, VideoFrame};

#[derive(Debug, Clone)]
pub enum Frame {
	Audio(AudioFrame),
	Video(VideoFrame),
}
