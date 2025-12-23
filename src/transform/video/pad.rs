use crate::core::Frame;
use crate::io::IoResult;

pub struct Pad {
	src_width: u32,
	src_height: u32,
	dst_width: u32,
	dst_height: u32,
	x: u32,
	y: u32,
	fill_y: u8,
	fill_u: u8,
	fill_v: u8,
}

impl Pad {
	pub fn new(
		src_width: u32,
		src_height: u32,
		dst_width: u32,
		dst_height: u32,
		x: u32,
		y: u32,
	) -> Self {
		Self {
			src_width,
			src_height,
			dst_width,
			dst_height,
			x,
			y,
			fill_y: 16,
			fill_u: 128,
			fill_v: 128,
		}
	}

	pub fn center(src_width: u32, src_height: u32, dst_width: u32, dst_height: u32) -> Self {
		let x = (dst_width.saturating_sub(src_width)) / 2;
		let y = (dst_height.saturating_sub(src_height)) / 2;
		Self::new(src_width, src_height, dst_width, dst_height, x, y)
	}

	pub fn with_color(mut self, y: u8, u: u8, v: u8) -> Self {
		self.fill_y = y;
		self.fill_u = u;
		self.fill_v = v;
		self
	}

	pub fn with_black(self) -> Self {
		self.with_color(16, 128, 128)
	}

	pub fn output_dimensions(&self) -> (u32, u32) {
		(self.dst_width, self.dst_height)
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let src_y_size = (self.src_width * self.src_height) as usize;
			let src_uv_size = src_y_size / 4;

			let src_y = &video_frame.data[0..src_y_size];
			let src_u = &video_frame.data[src_y_size..src_y_size + src_uv_size];
			let src_v = &video_frame.data[src_y_size + src_uv_size..src_y_size + 2 * src_uv_size];

			let dst_y_size = (self.dst_width * self.dst_height) as usize;
			let dst_uv_size = dst_y_size / 4;

			let mut dst_data = vec![self.fill_y; dst_y_size + 2 * dst_uv_size];

			{
				let (dst_y, dst_uv) = dst_data.split_at_mut(dst_y_size);
				let (dst_u, dst_v) = dst_uv.split_at_mut(dst_uv_size);

				dst_u.fill(self.fill_u);
				dst_v.fill(self.fill_v);

				self.copy_plane(
					src_y,
					dst_y,
					self.src_width,
					self.src_height,
					self.dst_width,
					self.x,
					self.y,
				);

				let src_uv_w = self.src_width / 2;
				let src_uv_h = self.src_height / 2;
				let dst_uv_w = self.dst_width / 2;
				let uv_x = self.x / 2;
				let uv_y = self.y / 2;

				self.copy_plane(src_u, dst_u, src_uv_w, src_uv_h, dst_uv_w, uv_x, uv_y);
				self.copy_plane(src_v, dst_v, src_uv_w, src_uv_h, dst_uv_w, uv_x, uv_y);
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

	fn copy_plane(
		&self,
		src: &[u8],
		dst: &mut [u8],
		src_w: u32,
		src_h: u32,
		dst_w: u32,
		x: u32,
		y: u32,
	) {
		for row in 0..src_h {
			for col in 0..src_w {
				let src_idx = (row * src_w + col) as usize;
				let dst_idx = ((y + row) * dst_w + (x + col)) as usize;

				if src_idx < src.len() && dst_idx < dst.len() {
					dst[dst_idx] = src[src_idx];
				}
			}
		}
	}
}
