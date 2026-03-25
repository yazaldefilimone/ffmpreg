use crate::Message;
use crate::container;
use crate::core::{Hint, Packet, Stream};
use crate::io::File;
use crate::message::Result;
use crate::{core::Muxer, io::Io};
use std::path::PathBuf;

pub struct Output {
	path: PathBuf,
	io: Option<Box<dyn Io>>,
	called: bool,
	muxer: Option<Box<dyn Muxer>>,
}

impl Output {
	fn ensure_file(&mut self) -> Result<()> {
		if self.io.is_none() {
			let io = File::create(&self.path)?;
			self.io = Some(Box::new(io));
		}
		Ok(())
	}

	fn ensure_muxer(&mut self, _stream: Option<&Stream>) -> Result<()> {
		if self.muxer.is_some() {
			return Ok(());
		}
		self.ensure_file()?;

		// auto infer based of codec?
		let extension = self.extension();
		let hint = Hint { extension, ..Default::default() };
		// todo: take io?
		let muxer = container::select_muxer(hint, self.io.take().unwrap())?;
		self.muxer = Some(muxer);
		Ok(())
	}

	fn muxer(&mut self, stream: Option<&Stream>) -> Result<&mut dyn Muxer> {
		self.ensure_muxer(stream)?;

		if let Some(muxer) = self.muxer.as_deref_mut() {
			return Ok(muxer);
		}

		Err(Message::Other("muxer not ready"))
	}

	fn extension(&self) -> Option<String> {
		self.path.extension().and_then(|e| e.to_str()).map(|s| s.to_string())
	}

	pub fn create(path: impl AsRef<std::path::Path>) -> Result<Self> {
		let path = path.as_ref().to_path_buf();
		Ok(Self { path, io: None, called: false, muxer: None })
	}
}

impl Muxer for Output {
	fn add(&mut self, stream: &Stream) -> Result<usize> {
		self.muxer(Some(stream))?.add(stream)
	}

	fn write(&mut self, packet: Packet) -> Result<()> {
		self.called = true;
		self.muxer(None)?.write(packet)
	}

	fn finish(&mut self) -> Result<()> {
		// todo: make sense?
		if !self.called {
			return Ok(());
		}
		self.muxer(None)?.finish()
	}
}
