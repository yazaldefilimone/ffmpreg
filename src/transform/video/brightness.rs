use crate::core::Frame;
use crate::io::IoResult;

pub struct Brightness {
	factor: f32,
	width: u32,
	height: u32,
}

impl Brightness {
	pub fn new(width: u32, height: u32, factor: f32) -> Self {
		Self { factor, width, height }
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let y_size = (self.width * self.height) as usize;
			let _uv_size = y_size / 4;

			let mut dst_data = video_frame.data.clone();

			for i in 0..y_size {
				let y = dst_data[i] as f32;
				let adjusted = (y + self.factor * 255.0).clamp(0.0, 255.0);
				dst_data[i] = adjusted as u8;
			}

			let new_video = crate::core::FrameVideo::new(
				dst_data,
				video_frame.width,
				video_frame.height,
				video_frame.format,
			);
			Ok(
				Frame::new_video(new_video, frame.timebase.clone(), frame.stream_index).with_pts(frame.pts),
			)
		} else {
			Ok(frame.clone())
		}
	}
}
