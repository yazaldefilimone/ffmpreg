pub mod audio;
pub mod video;

pub use audio::{AudioFrame, AudioParams, Channels, SampleFormat, SampleLayout};
pub use video::{Keyframe, Pixel, VideoFrame};

#[derive(Debug, Clone)]
pub enum Frame {
	Audio(AudioFrame),
	Video(VideoFrame),
}
