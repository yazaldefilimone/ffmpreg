use crate::Message;
use crate::io::Io;
use crate::message::Result;
use std::io::{Error, Read};

pub struct HttpIo {
	response: reqwest::blocking::Response,
	cache: Vec<u8>,
	cursor: usize,
	content_length: Option<u64>,
}

impl HttpIo {
	pub fn open(url: &str) -> Result<Self> {
		let map_err = |e| Message::Io(Error::other(format!("{}", e)));
		let response = reqwest::blocking::get(url).map_err(map_err)?;
		if !response.status().is_success() {
			return Err(Message::Other("http request failed"));
		}

		let content_length = response.content_length();
		Ok(Self { response, cache: Vec::new(), cursor: 0, content_length })
	}

	fn fill_to(&mut self, target: usize) -> Result<()> {
		while self.cache.len() < target {
			let missing = target - self.cache.len();
			let chunk_size = missing.min(8192);
			let mut chunk = vec![0u8; chunk_size];
			let n = self.response.read(&mut chunk)?;
			if n == 0 {
				break;
			}
			chunk.truncate(n);
			self.cache.extend_from_slice(&chunk);
		}
		Ok(())
	}
}

impl Io for HttpIo {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let target = self.cursor.saturating_add(buf.len());
		self.fill_to(target)?;

		if self.cursor >= self.cache.len() {
			return Ok(0);
		}

		let available = &self.cache[self.cursor..];
		let n = available.len().min(buf.len());
		buf[..n].copy_from_slice(&available[..n]);
		self.cursor += n;
		Ok(n)
	}

	fn seek(&mut self, position: u64) -> Result<()> {
		let position = usize::try_from(position).map_err(|_| Message::Other("http seek overflow"))?;
		self.fill_to(position)?;
		if position > self.cache.len() {
			return Err(Message::Other("cannot seek past downloaded http data"));
		}
		self.cursor = position;
		Ok(())
	}

	fn size(&self) -> Result<u64> {
		match self.content_length {
			Some(len) => Ok(len),
			None => Err("content-length unknown".into()),
		}
	}

	fn write(&mut self, _: &[u8]) -> Result<usize> {
		Err("write not supported on http".into())
	}
}
