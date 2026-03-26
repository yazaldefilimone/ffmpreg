use rustc_hash::FxHashMap;

use crate::core::{
	Context, Metadata,
	time::{Time, TimeBase},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(pub usize);

impl From<usize> for StreamId {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

pub type StreamHashMap = FxHashMap<StreamId, Context>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
	Audio,
	Video,
	Subtitle,
	Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodecId {
	pub id: String,
}

impl CodecId {
	pub fn new(id: &str) -> Self {
		Self { id: id.to_string() }
	}
}

#[derive(Debug, Clone)]
pub struct Profile {
	pub name: String,
	pub level: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Parameters {
	pub codec: CodecId,
	pub profile: Option<Profile>,
	pub bitrate: Option<u32>,
	pub sample_rate: Option<u32>,
	pub channels: Option<u8>,
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub pixel_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Stream {
	pub id: StreamId,
	pub kind: StreamKind,
	pub time_base: TimeBase,
	pub duration: Option<Time>,
	pub parameters: Parameters,
	pub metadata: Metadata,
}

#[derive(Debug, Clone, Default)]
pub struct StreamSet {
	pub streams: Vec<Stream>,
}

impl StreamSet {
	pub fn new(streams: Vec<Stream>) -> Self {
		Self { streams }
	}

	pub fn iter(&self) -> impl Iterator<Item = &Stream> {
		self.streams.iter()
	}

	pub fn get(&self, id: StreamId) -> Option<&Stream> {
		self.streams.iter().find(|s| s.id == id)
	}

	pub fn by_kind(&self, kind: StreamKind) -> Vec<&Stream> {
		self.streams.iter().filter(|s| s.kind == kind).collect()
	}

	pub fn add(&mut self, stream: Stream) {
		self.streams.push(stream);
	}
}

impl Stream {
	pub fn video(id: StreamId, width: u32, height: u32, pixel_format: &str, codec: CodecId) -> Self {
		Self {
			id,
			kind: StreamKind::Video,
			time_base: TimeBase::new(1, 30),
			duration: None,
			parameters: Parameters {
				codec,
				profile: None,
				bitrate: None,
				sample_rate: None,
				channels: None,
				width: Some(width),
				height: Some(height),
				pixel_format: Some(pixel_format.to_string()),
			},
			metadata: Metadata::default(),
		}
	}

	pub fn audio(id: StreamId, sample_rate: u32, channels: u8, codec: CodecId) -> Self {
		Self {
			id,
			kind: StreamKind::Audio,
			time_base: TimeBase::new(1, sample_rate),
			duration: None,
			parameters: Parameters {
				codec,
				profile: None,
				bitrate: None,
				sample_rate: Some(sample_rate),
				channels: Some(channels),
				width: None,
				height: None,
				pixel_format: None,
			},
			metadata: Metadata::default(),
		}
	}
}
