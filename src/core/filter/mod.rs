use crate::core::Selector;
use crate::core::frame::Frame;
use crate::core::resampler::pcm::PcmResampler;
use crate::core::traits::Transform;
use crate::message::Result;
use rustc_hash::FxHashMap;

pub struct FilterGraph {
	filters: FxHashMap<usize, Vec<Box<dyn Transform>>>,
	global_filters: Vec<Box<dyn Transform>>,
}

impl FilterGraph {
	pub fn new() -> Self {
		Self { filters: FxHashMap::default(), global_filters: Vec::new() }
	}

	pub fn add(&mut self, selector: Selector, transform: Box<dyn Transform>) {
		match selector {
			Selector::Id(track_id) => {
				self.filters.entry(track_id).or_default().push(transform);
			}
			Selector::All => self.global_filters.push(transform),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.filters.is_empty()
	}

	pub fn apply(&mut self, mut frame: Frame) -> Result<Frame> {
		let track_filters = self.filters.get_mut(&frame.track_id);
		let has_track = track_filters.as_ref().is_some_and(|t| !t.is_empty());
		let has_global = !self.global_filters.is_empty();

		if !has_track && !has_global {
			return Ok(frame);
		}

		let audio = match frame.audio_mut() {
			Some(a) => a,
			None => return Ok(frame),
		};

		let bit_depth = audio.bit_depth;
		let pcm = PcmResampler::new(bit_depth, bit_depth);
		let mut samples = pcm.decode(&audio.data)?;

		if let Some(filters) = track_filters {
			for transform in filters.iter_mut() {
				transform.apply(&mut samples)?;
			}
		}

		for transform in self.global_filters.iter_mut() {
			transform.apply(&mut samples)?;
		}

		audio.data = pcm.encode(&samples)?;

		Ok(frame)
	}
}
