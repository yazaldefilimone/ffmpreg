use crate::cli::{PlayArgs, ProbeArgs, RunArgs};
use crate::core::Selector;
use crate::{error, message};
use rustc_hash::FxHashMap;
use std::fmt::Debug;

pub struct StreamOption {
	pub selector: Selector,
	pub codec: Option<String>,
	pub kind: StreamKind,
	pub table: FxHashMap<String, StreamValue>,
}

impl StreamOption {
	pub fn default(kind: StreamKind) -> Self {
		let table = FxHashMap::default();
		let selector = Selector::Id(0);
		StreamOption { selector, codec: None, kind, table }
	}

	pub fn from_raw(raw: &str, kind: StreamKind) -> message::Result<Self> {
		let mut selector = Selector::All;
		let mut codec = None;
		let mut table = FxHashMap::default();

		for part in raw.split_whitespace() {
			let mut it = part.splitn(2, '=');
			let key = it.next().unwrap();
			let value = it.next().unwrap_or("true");

			match key {
				"track" | "t" => {
					selector = match value {
						"all" | "*" => Selector::All,
						id => {
							let id = id.parse::<usize>().map_err(|_| error!("invalid track id '{}'", id))?;
							Selector::Id(id)
						}
					};
				}
				"codec" => {
					codec = Some(value.to_string());
				}
				_ => {
					let sv = StreamValue::parse(value);
					table.insert(key.to_string(), sv);
				}
			}
		}

		Ok(StreamOption { selector, codec, kind, table })
	}
}

pub fn parse_stream_options(
	audio: &[String],
	video: &[String],
	subtitle: &[String],
) -> message::Result<Vec<StreamOption>> {
	let mut options = Vec::new();
	for raw in audio {
		options.push(StreamOption::from_raw(raw, StreamKind::Audio)?);
	}

	for raw in video {
		options.push(StreamOption::from_raw(raw, StreamKind::Video)?);
	}

	for raw in subtitle {
		options.push(StreamOption::from_raw(raw, StreamKind::Subtitle)?);
	}

	if audio.is_empty() {
		options.push(StreamOption::default(StreamKind::Audio));
	}
	if video.is_empty() {
		options.push(StreamOption::default(StreamKind::Video));
	}

	Ok(options)
}

impl Debug for StreamOption {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StreamOption")
			.field("selector", &self.selector)
			.field("kind", &self.kind)
			.finish()
	}
}

#[derive(Debug)]
pub enum StreamKind {
	Audio,
	Video,
	Subtitle,
}

pub enum StreamValue {
	String(String),
	Integer(usize),
	Float(f64),
	Boolean(bool),
}

impl StreamValue {
	pub fn parse(value: &str) -> Self {
		if let Ok(i) = value.parse::<usize>() {
			return StreamValue::Integer(i);
		}
		if let Ok(f) = value.parse::<f64>() {
			return StreamValue::Float(f);
		}
		if value.is_empty() {
			return StreamValue::Boolean(true);
		}
		match value {
			"true" => StreamValue::Boolean(true),
			"false" => StreamValue::Boolean(false),
			_ => StreamValue::String(value.to_string()),
		}
	}
}

impl From<&str> for StreamValue {
	fn from(s: &str) -> Self {
		StreamValue::String(s.to_string())
	}
}

impl From<i64> for StreamValue {
	fn from(i: i64) -> Self {
		StreamValue::Integer(i as usize)
	}
}

impl From<f64> for StreamValue {
	fn from(f: f64) -> Self {
		StreamValue::Float(f)
	}
}

impl From<bool> for StreamValue {
	fn from(b: bool) -> Self {
		StreamValue::Boolean(b)
	}
}
impl From<String> for StreamValue {
	fn from(s: String) -> Self {
		StreamValue::String(s)
	}
}

impl From<usize> for StreamValue {
	fn from(i: usize) -> Self {
		StreamValue::Integer(i)
	}
}

impl From<f32> for StreamValue {
	fn from(f: f32) -> Self {
		StreamValue::Float(f as f64)
	}
}

impl RunArgs {
	pub fn stream_options(&self) -> message::Result<Vec<StreamOption>> {
		parse_stream_options(&self.audio, &self.video, &self.subtitle)
	}
}

impl ProbeArgs {
	pub fn stream_options(&self) -> message::Result<Vec<StreamOption>> {
		parse_stream_options(&self.audio, &self.video, &self.subtitle)
	}
}

impl PlayArgs {
	pub fn stream_options(&self) -> message::Result<Vec<StreamOption>> {
		parse_stream_options(&self.audio, &self.video, &self.subtitle)
	}
}
