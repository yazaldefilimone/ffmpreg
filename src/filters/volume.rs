use crate::core::traits::Transform;
use crate::message::Result;

pub struct Volume {
	factor: f32,
}

impl Volume {
	pub fn new(factor: f32) -> Self {
		Self { factor }
	}
}

impl Transform for Volume {
	fn apply(&mut self, samples: &mut [f32]) -> Result<()> {
		for s in samples.iter_mut() {
			*s = (*s * self.factor).clamp(-1.0, 1.0);
		}
		Ok(())
	}

	fn name(&self) -> &'static str {
		"volume"
	}
}
