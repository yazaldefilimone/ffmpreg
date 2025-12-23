use crate::core::Frame;
use crate::io::IoResult;

pub struct Blur {
	width: u32,
	height: u32,
	radius: u32,
}

impl Blur {
	pub fn new(width: u32, height: u32, radius: u32) -> Self {
		Self { width, height, radius }
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

			self.box_blur(src_y, dst_y, self.width, self.height);

			let uv_w = self.width / 2;
			let uv_h = self.height / 2;
			self.box_blur(src_u, dst_u, uv_w, uv_h);
			self.box_blur(src_v, dst_v, uv_w, uv_h);

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

	fn box_blur(&self, src: &[u8], dst: &mut [u8], width: u32, height: u32) {
		let r = self.radius as i32;

		for y in 0..height as i32 {
			for x in 0..width as i32 {
				let mut sum: u32 = 0;
				let mut count: u32 = 0;

				for dy in -r..=r {
					for dx in -r..=r {
						let nx = x + dx;
						let ny = y + dy;

						if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
							let idx = (ny as u32 * width + nx as u32) as usize;
							if idx < src.len() {
								sum += src[idx] as u32;
								count += 1;
							}
						}
					}
				}

				let dst_idx = (y as u32 * width + x as u32) as usize;
				if dst_idx < dst.len() && count > 0 {
					dst[dst_idx] = (sum / count) as u8;
				}
			}
		}
	}
}
