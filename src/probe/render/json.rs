use crate::error;
use crate::message::Result;
use crate::probe::model;

#[allow(dead_code)]
pub fn render_raw(media: &model::MediaFile) -> Result<String> {
	serde_json::to_string(media).map_err(|err| error!("json render failed: {}", err))
}

#[allow(dead_code)]
pub fn render_pretty(media: &model::MediaFile) -> Result<String> {
	serde_json::to_string_pretty(media).map_err(|err| error!("json render failed: {}", err))
}
