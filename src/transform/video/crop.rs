use crate::core::Frame;
use crate::io::IoResult;

pub struct Crop {
	src_width: u32,
	src_height: u32,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
}

impl Crop {
	pub fn new(src_width: u32, src_height: u32, x: u32, y: u32, width: u32, height: u32) -> Self {
		let x = x.min(src_width);
		let y = y.min(src_height);
		let width = width.min(src_width - x);
		let height = height.min(src_height - y);
		Self { src_width, src_height, x, y, width, height }
	}

	pub fn center(src_width: u32, src_height: u32, width: u32, height: u32) -> Self {
		let x = (src_width.saturating_sub(width)) / 2;
		let y = (src_height.saturating_sub(height)) / 2;
		Self::new(src_width, src_height, x, y, width, height)
	}

	pub fn output_dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let src_y_size = (self.src_width * self.src_height) as usize;
			let src_uv_size = src_y_size / 4;

			let src_y = &video_frame.data[0..src_y_size];
			let src_u = &video_frame.data[src_y_size..src_y_size + src_uv_size];
			let src_v = &video_frame.data[src_y_size + src_uv_size..src_y_size + 2 * src_uv_size];

			let dst_y_size = (self.width * self.height) as usize;
			let dst_uv_size = dst_y_size / 4;

			let mut dst_data = vec![0u8; dst_y_size + 2 * dst_uv_size];
			let (dst_y, dst_uv) = dst_data.split_at_mut(dst_y_size);
			let (dst_u, dst_v) = dst_uv.split_at_mut(dst_uv_size);

			self.crop_plane(src_y, dst_y, self.src_width, self.x, self.y, self.width, self.height);

			let uv_x = self.x / 2;
			let uv_y = self.y / 2;
			let uv_w = self.width / 2;
			let uv_h = self.height / 2;
			let src_uv_w = self.src_width / 2;

			self.crop_plane(src_u, dst_u, src_uv_w, uv_x, uv_y, uv_w, uv_h);
			self.crop_plane(src_v, dst_v, src_uv_w, uv_x, uv_y, uv_w, uv_h);

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

	fn crop_plane(
		&self,
		src: &[u8],
		dst: &mut [u8],
		src_w: u32,
		x: u32,
		y: u32,
		width: u32,
		height: u32,
	) {
		for row in 0..height {
			for col in 0..width {
				let src_idx = ((y + row) * src_w + (x + col)) as usize;
				let dst_idx = (row * width + col) as usize;

				if src_idx < src.len() && dst_idx < dst.len() {
					dst[dst_idx] = src[src_idx];
				}
			}
		}
	}
}
