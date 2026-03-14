use crate::core::frame::BitDepth;
use crate::core::time::Timestamp;
use crate::core::track::{ColorSpace, Pixel, VideoFormat};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
	YUV420,
	YUV422,
	YUV444,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyframe {
	Key,
	NonKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFormat {
	pub depth: u8, // bits per channel
	pub format: Format,
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
	y: Vec<u8>,
	u: Vec<u8>,
	v: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub keyframe: Keyframe,
	pub pixel: PixelFormat,
	pub pts: Timestamp,
}

impl PixelFormat {
	pub fn new(depth: u8, format: Format) -> Self {
		Self { depth, format }
	}

	pub fn yuv420(depth: u8) -> Self {
		Self { depth, format: Format::YUV420 }
	}

	pub fn yuv422(depth: u8) -> Self {
		Self { depth, format: Format::YUV422 }
	}

	pub fn yuv444(depth: u8) -> Self {
		Self { depth, format: Format::YUV444 }
	}

	pub fn bytes_per_component(&self) -> usize {
		(self.depth as usize + 7) / 8
	}

	pub fn plane_sizes(&self, width: u32, height: u32) -> [usize; 3] {
		let bpc = self.bytes_per_component();
		let w = width as usize;
		let h = height as usize;
		match self.format {
			Format::YUV444 => {
				let plane = w * h * bpc;
				[plane, plane, plane]
			}
			Format::YUV422 => {
				let y_size = w * h * bpc;
				let chroma_w = (w + 1) / 2;
				let uv_size = chroma_w * h * bpc;
				[y_size, uv_size, uv_size]
			}
			Format::YUV420 => {
				let y_size = w * h * bpc;
				let chroma_w = (w + 1) / 2;
				let chroma_h = (h + 1) / 2;
				let uv_size = chroma_w * chroma_h * bpc;
				[y_size, uv_size, uv_size]
			}
		}
	}

	pub fn total_size(&self, width: u32, height: u32) -> usize {
		let sizes = self.plane_sizes(width, height);
		sizes[0] + sizes[1] + sizes[2]
	}
}

impl VideoFrame {
	pub fn new(
		y: Vec<u8>,
		u: Vec<u8>,
		v: Vec<u8>,
		width: u32,
		height: u32,
		keyframe: Keyframe,
		pixel: PixelFormat,
	) -> Self {
		Self { y, u, v, width, height, keyframe, pixel, pts: Timestamp::default() }
	}

	pub fn new_empty(width: u32, height: u32, keyframe: Keyframe, pixel: PixelFormat) -> Self {
		let sizes = pixel.plane_sizes(width, height);
		Self {
			y: Vec::with_capacity(sizes[0]),
			u: Vec::with_capacity(sizes[1]),
			v: Vec::with_capacity(sizes[2]),
			width,
			height,
			keyframe,
			pixel,
			pts: Timestamp::default(),
		}
	}

	pub fn new_zeroed(width: u32, height: u32, keyframe: Keyframe, pixel: PixelFormat) -> Self {
		let sizes = pixel.plane_sizes(width, height);
		Self {
			y: vec![0u8; sizes[0]],
			u: vec![0u8; sizes[1]],
			v: vec![0u8; sizes[2]],
			width,
			height,
			keyframe,
			pixel,
			pts: Timestamp::default(),
		}
	}

	pub fn from_interleaved(
		data: &[u8],
		width: u32,
		height: u32,
		keyframe: Keyframe,
		pixel: PixelFormat,
	) -> Self {
		let sizes = pixel.plane_sizes(width, height);
		let total = sizes[0] + sizes[1] + sizes[2];
		let clamped = if data.len() < total { data.len() } else { total };
		let mut offset = 0;

		let y_end = sizes[0].min(clamped);
		let y = data[offset..y_end].to_vec();
		offset = y_end;

		let u_end = (offset + sizes[1]).min(clamped);
		let u = data[offset..u_end].to_vec();
		offset = u_end;

		let v_end = (offset + sizes[2]).min(clamped);
		let v = data[offset..v_end].to_vec();

		Self { y, u, v, width, height, keyframe, pixel, pts: Timestamp::default() }
	}

	pub fn with_pts(mut self, pts: Timestamp) -> Self {
		self.pts = pts;
		self
	}

	pub fn with_keyframe(mut self, keyframe: Keyframe) -> Self {
		self.keyframe = keyframe;
		self
	}

	// plane accessors

	pub fn y_plane(&self) -> &[u8] {
		&self.y
	}

	pub fn u_plane(&self) -> &[u8] {
		&self.u
	}

	pub fn v_plane(&self) -> &[u8] {
		&self.v
	}

	pub fn y_plane_mut(&mut self) -> &mut Vec<u8> {
		&mut self.y
	}

	pub fn u_plane_mut(&mut self) -> &mut Vec<u8> {
		&mut self.u
	}

	pub fn v_plane_mut(&mut self) -> &mut Vec<u8> {
		&mut self.v
	}

	pub fn planes(&self) -> (&[u8], &[u8], &[u8]) {
		(&self.y, &self.u, &self.v)
	}

	pub fn planes_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<u8>, &mut Vec<u8>) {
		(&mut self.y, &mut self.u, &mut self.v)
	}

	pub fn set_planes(&mut self, y: Vec<u8>, u: Vec<u8>, v: Vec<u8>) {
		self.y = y;
		self.u = u;
		self.v = v;
	}

	pub fn to_interleaved(&self) -> Vec<u8> {
		let total = self.y.len() + self.u.len() + self.v.len();
		let mut data = Vec::with_capacity(total);
		data.extend_from_slice(&self.y);
		data.extend_from_slice(&self.u);
		data.extend_from_slice(&self.v);
		data
	}

	pub fn size(&self) -> usize {
		self.y.len() + self.u.len() + self.v.len()
	}

	pub fn expected_size(&self) -> usize {
		self.pixel.total_size(self.width, self.height)
	}

	pub fn is_valid(&self) -> bool {
		let sizes = self.pixel.plane_sizes(self.width, self.height);
		self.y.len() == sizes[0] && self.u.len() == sizes[1] && self.v.len() == sizes[2]
	}

	pub fn is_key(&self) -> bool {
		matches!(self.keyframe, Keyframe::Key)
	}

	pub fn duration_seconds(&self, fps: f64) -> f64 {
		1.0 / fps
	}

	pub fn video_format(&self) -> VideoFormat {
		VideoFormat {
			width: self.width,
			height: self.height,
			pixel: Pixel {
				depth: BitDepth::from_bits_any(self.pixel.depth),
				color_space: ColorSpace::YUV,
			},
		}
	}
}

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::YUV420 => write!(f, "YUV420"),
			Self::YUV422 => write!(f, "YUV422"),
			Self::YUV444 => write!(f, "YUV444"),
		}
	}
}

impl fmt::Display for Keyframe {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Key => write!(f, "Key"),
			Self::NonKey => write!(f, "Non-Key"),
		}
	}
}

impl fmt::Display for PixelFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}-bit {}", self.depth, self.format)
	}
}
