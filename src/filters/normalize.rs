use crate::core::traits::Transform;
use crate::message::Result;

pub struct Normalize {
	target_peak: f32,
}

impl Normalize {
	pub fn new() -> Self {
		Self { target_peak: 1.0 }
	}

	pub fn with_target(target_peak: f32) -> Self {
		Self { target_peak }
	}
}

impl Transform for Normalize {
	fn apply(&mut self, samples: &mut [f32]) -> Result<()> {
		let peak = samples.iter().fold(0.0f32, |max, &s| max.max(s.abs()));

		if peak > 0.0 && peak != self.target_peak {
			let gain = self.target_peak / peak;
			for s in samples.iter_mut() {
				*s = (*s * gain).clamp(-1.0, 1.0);
			}
		}

		Ok(())
	}

	fn name(&self) -> &'static str {
		"normalize"
	}
}
