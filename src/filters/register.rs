use crate::core::resolver;
use crate::core::traits::Transform;
use crate::filters::{Normalize, Volume};
use crate::{error, message};
use rustc_hash::FxHashMap;

pub fn register_transforms() -> resolver::TransformMapper {
	let mut mapper: resolver::TransformMapper = FxHashMap::default();

	mapper.insert("volume", create_volume);
	mapper.insert("normalize", create_normalize);
	mapper
}

fn create_volume(value: &str) -> message::Result<Box<dyn Transform>> {
	let factor = value.parse::<f32>().map_err(|_| error!("unsupported volume '{}'", value))?;
	Ok(Box::new(Volume::new(factor)))
}

fn create_normalize(value: &str) -> message::Result<Box<dyn Transform>> {
	if value.is_empty() || value == "true" {
		return Ok(Box::new(Normalize::new()));
	}
	let target = value.parse::<f32>().map_err(|_| error!("unsupported normalize '{}'", value))?;
	Ok(Box::new(Normalize::with_target(target)))
}
