use crate::core::Frame;
use crate::io::IoResult;

#[derive(Debug, Clone, Copy)]
pub enum ScaleMode {
	NearestNeighbor,
	Bilinear,
}

pub struct Scale {
	target_width: u32,
	target_height: u32,
	src_width: u32,
	src_height: u32,
	mode: ScaleMode,
}

impl Scale {
	pub fn new(src_width: u32, src_height: u32, target_width: u32, target_height: u32) -> Self {
		Self { target_width, target_height, src_width, src_height, mode: ScaleMode::Bilinear }
	}

	pub fn with_mode(mut self, mode: ScaleMode) -> Self {
		self.mode = mode;
		self
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let src_y_size = (self.src_width * self.src_height) as usize;
			let src_uv_size = src_y_size / 4;

			let dst_y_size = (self.target_width * self.target_height) as usize;
			let dst_uv_size = dst_y_size / 4;

			let src_y = &video_frame.data[0..src_y_size];
			let src_u = &video_frame.data[src_y_size..src_y_size + src_uv_size];
			let src_v = &video_frame.data[src_y_size + src_uv_size..src_y_size + 2 * src_uv_size];

			let mut dst_data = vec![0u8; dst_y_size + 2 * dst_uv_size];
			let (dst_y, dst_uv) = dst_data.split_at_mut(dst_y_size);
			let (dst_u, dst_v) = dst_uv.split_at_mut(dst_uv_size);

			self.scale_plane(
				src_y,
				dst_y,
				self.src_width,
				self.src_height,
				self.target_width,
				self.target_height,
			);

			let src_uv_w = self.src_width / 2;
			let src_uv_h = self.src_height / 2;
			let dst_uv_w = self.target_width / 2;
			let dst_uv_h = self.target_height / 2;

			self.scale_plane(src_u, dst_u, src_uv_w, src_uv_h, dst_uv_w, dst_uv_h);
			self.scale_plane(src_v, dst_v, src_uv_w, src_uv_h, dst_uv_w, dst_uv_h);

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

	fn scale_plane(
		&self,
		src: &[u8],
		dst: &mut [u8],
		src_w: u32,
		src_h: u32,
		dst_w: u32,
		dst_h: u32,
	) {
		match self.mode {
			ScaleMode::NearestNeighbor => self.scale_nearest(src, dst, src_w, src_h, dst_w, dst_h),
			ScaleMode::Bilinear => self.scale_bilinear(src, dst, src_w, src_h, dst_w, dst_h),
		}
	}

	fn scale_nearest(
		&self,
		src: &[u8],
		dst: &mut [u8],
		src_w: u32,
		src_h: u32,
		dst_w: u32,
		dst_h: u32,
	) {
		let x_ratio = src_w as f64 / dst_w as f64;
		let y_ratio = src_h as f64 / dst_h as f64;

		for y in 0..dst_h {
			for x in 0..dst_w {
				let src_x = (x as f64 * x_ratio) as u32;
				let src_y = (y as f64 * y_ratio) as u32;
				let src_idx = (src_y * src_w + src_x) as usize;
				let dst_idx = (y * dst_w + x) as usize;

				if src_idx < src.len() && dst_idx < dst.len() {
					dst[dst_idx] = src[src_idx];
				}
			}
		}
	}

	fn scale_bilinear(
		&self,
		src: &[u8],
		dst: &mut [u8],
		src_w: u32,
		src_h: u32,
		dst_w: u32,
		dst_h: u32,
	) {
		let x_ratio = (src_w as f64 - 1.0) / (dst_w as f64 - 1.0).max(1.0);
		let y_ratio = (src_h as f64 - 1.0) / (dst_h as f64 - 1.0).max(1.0);

		for y in 0..dst_h {
			for x in 0..dst_w {
				let src_x = x as f64 * x_ratio;
				let src_y = y as f64 * y_ratio;

				let x0 = src_x.floor() as u32;
				let y0 = src_y.floor() as u32;
				let x1 = (x0 + 1).min(src_w - 1);
				let y1 = (y0 + 1).min(src_h - 1);

				let x_frac = src_x - x0 as f64;
				let y_frac = src_y - y0 as f64;

				let get_pixel = |px: u32, py: u32| -> f64 {
					let idx = (py * src_w + px) as usize;
					if idx < src.len() { src[idx] as f64 } else { 0.0 }
				};

				let p00 = get_pixel(x0, y0);
				let p10 = get_pixel(x1, y0);
				let p01 = get_pixel(x0, y1);
				let p11 = get_pixel(x1, y1);

				let top = p00 * (1.0 - x_frac) + p10 * x_frac;
				let bottom = p01 * (1.0 - x_frac) + p11 * x_frac;
				let value = top * (1.0 - y_frac) + bottom * y_frac;

				let dst_idx = (y * dst_w + x) as usize;
				if dst_idx < dst.len() {
					dst[dst_idx] = value.clamp(0.0, 255.0) as u8;
				}
			}
		}
	}
}
