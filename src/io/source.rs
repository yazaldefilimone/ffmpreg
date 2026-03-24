use crate::Result;

pub enum Source {
	File(String),
	Url(String),
}

pub fn parse_source(input: &str) -> Result<Source> {
	if let Some(rest) = input.strip_prefix("http://") {
		return Ok(Source::Url(format!("http://{}", rest)));
	}

	if let Some(rest) = input.strip_prefix("https://") {
		return Ok(Source::Url(format!("https://{}", rest)));
	}

	// valida path simples
	if input.contains("://") {
		return Err("unsupported scheme".into());
	}

	Ok(Source::File(input.to_string()))
}
