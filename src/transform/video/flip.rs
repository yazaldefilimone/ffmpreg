use crate::core::Frame;
use crate::io::IoResult;

#[derive(Debug, Clone, Copy)]
pub enum FlipDirection {
	Horizontal,
	Vertical,
}

pub struct Flip {
	direction: FlipDirection,
	width: u32,
	height: u32,
}

impl Flip {
	pub fn new(width: u32, height: u32, direction: FlipDirection) -> Self {
		Self { direction, width, height }
	}

	pub fn horizontal(width: u32, height: u32) -> Self {
		Self::new(width, height, FlipDirection::Horizontal)
	}

	pub fn vertical(width: u32, height: u32) -> Self {
		Self::new(width, height, FlipDirection::Vertical)
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let y_size = (self.width * self.height) as usize;
			let uv_size = y_size / 4;

			let src_y = &video_frame.data[0..y_size];
			let src_u = &video_frame.data[y_size..y_size + uv_size];
			let src_v = &video_frame.data[y_size + uv_size..y_size + 2 * uv_size];

			let mut dst_data = vec![0u8; y_size + 2 * uv_size];
			let (dst_y, dst_uv) = dst_data.split_at_mut(y_size);
			let (dst_u, dst_v) = dst_uv.split_at_mut(uv_size);

			self.flip_plane(src_y, dst_y, self.width, self.height);

			let uv_w = self.width / 2;
			let uv_h = self.height / 2;
			self.flip_plane(src_u, dst_u, uv_w, uv_h);
			self.flip_plane(src_v, dst_v, uv_w, uv_h);

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

	fn flip_plane(&self, src: &[u8], dst: &mut [u8], width: u32, height: u32) {
		for y in 0..height {
			for x in 0..width {
				let src_idx = (y * width + x) as usize;
				let (dst_x, dst_y) = match self.direction {
					FlipDirection::Horizontal => (width - 1 - x, y),
					FlipDirection::Vertical => (x, height - 1 - y),
				};
				let dst_idx = (dst_y * width + dst_x) as usize;

				if src_idx < src.len() && dst_idx < dst.len() {
					dst[dst_idx] = src[src_idx];
				}
			}
		}
	}
}
