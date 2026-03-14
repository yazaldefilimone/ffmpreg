use crate::{error, message, utils::kv::KeyValue};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Selector {
	#[default]
	All,
	Id(usize),
}

impl Selector {
	pub fn matches(&self, id: usize) -> bool {
		match self {
			Selector::All => true,
			Selector::Id(selector) => id == *selector,
		}
	}

	pub fn all_selected(&self) -> bool {
		matches!(self, Selector::All)
	}

	pub fn from_kv(kv: &KeyValue) -> message::Result<Option<Self>> {
		let selector = match kv.get("track").or(kv.get("t")) {
			None => return Ok(None),
			Some(track) => match track.as_str() {
				"all" | "*" => Some(Selector::All),
				track => Some(Selector::Id(Self::parse_id(track)?)),
			},
		};
		Ok(selector)
	}

	fn parse_id(track: &str) -> message::Result<usize> {
		let value = track.parse().map_err(|_| error!("unable to parse track '{}'", track))?;
		Ok(value)
	}
}

impl From<usize> for Selector {
	fn from(id: usize) -> Self {
		Selector::Id(id)
	}
}

impl From<Selector> for Vec<usize> {
	fn from(selector: Selector) -> Self {
		match selector {
			Selector::All => Vec::new(),
			Selector::Id(id) => vec![id],
		}
	}
}

impl Display for Selector {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Selector::All => write!(f, "all"),
			Selector::Id(id) => write!(f, "{}", id),
		}
	}
}
