use crate::core::Frame;
use crate::io::IoResult;

#[derive(Debug, Clone, Copy)]
pub enum RotateAngle {
	Rotate90,
	Rotate180,
	Rotate270,
}

pub struct Rotate {
	angle: RotateAngle,
	width: u32,
	height: u32,
}

impl Rotate {
	pub fn new(width: u32, height: u32, angle: RotateAngle) -> Self {
		Self { angle, width, height }
	}

	pub fn rotate_90(width: u32, height: u32) -> Self {
		Self::new(width, height, RotateAngle::Rotate90)
	}

	pub fn rotate_180(width: u32, height: u32) -> Self {
		Self::new(width, height, RotateAngle::Rotate180)
	}

	pub fn rotate_270(width: u32, height: u32) -> Self {
		Self::new(width, height, RotateAngle::Rotate270)
	}

	pub fn output_dimensions(&self) -> (u32, u32) {
		match self.angle {
			RotateAngle::Rotate90 | RotateAngle::Rotate270 => (self.height, self.width),
			RotateAngle::Rotate180 => (self.width, self.height),
		}
	}

	pub fn apply_yuv420(&self, frame: &Frame) -> IoResult<Frame> {
		if let Some(video_frame) = frame.video() {
			let y_size = (self.width * self.height) as usize;
			let uv_size = y_size / 4;

			let src_y = &video_frame.data[0..y_size];
			let src_u = &video_frame.data[y_size..y_size + uv_size];
			let src_v = &video_frame.data[y_size + uv_size..y_size + 2 * uv_size];

			let (dst_w, dst_h) = self.output_dimensions();
			let dst_y_size = (dst_w * dst_h) as usize;
			let dst_uv_size = dst_y_size / 4;

			let mut dst_data = vec![0u8; dst_y_size + 2 * dst_uv_size];
			let (dst_y, dst_uv) = dst_data.split_at_mut(dst_y_size);
			let (dst_u, dst_v) = dst_uv.split_at_mut(dst_uv_size);

			self.rotate_plane(src_y, dst_y, self.width, self.height);

			let src_uv_w = self.width / 2;
			let src_uv_h = self.height / 2;
			self.rotate_plane(src_u, dst_u, src_uv_w, src_uv_h);
			self.rotate_plane(src_v, dst_v, src_uv_w, src_uv_h);

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

	fn rotate_plane(&self, src: &[u8], dst: &mut [u8], src_w: u32, src_h: u32) {
		let (dst_w, _dst_h) = match self.angle {
			RotateAngle::Rotate90 | RotateAngle::Rotate270 => (src_h, src_w),
			RotateAngle::Rotate180 => (src_w, src_h),
		};

		for y in 0..src_h {
			for x in 0..src_w {
				let src_idx = (y * src_w + x) as usize;
				let (dst_x, dst_y) = match self.angle {
					RotateAngle::Rotate90 => (src_h - 1 - y, x),
					RotateAngle::Rotate180 => (src_w - 1 - x, src_h - 1 - y),
					RotateAngle::Rotate270 => (y, src_w - 1 - x),
				};
				let dst_idx = (dst_y * dst_w + dst_x) as usize;

				if src_idx < src.len() && dst_idx < dst.len() {
					dst[dst_idx] = src[src_idx];
				}
			}
		}
	}
}
