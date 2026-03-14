use crate::core::packet::Packet;
use crate::core::resolver::ContainerResolver;
use crate::core::track::Metadata;
use crate::core::{Muxer, track::Format};
use crate::io::File;
use crate::message::Result;
use crate::utils;
use std::path::{Path, PathBuf};

pub struct OutputBuilder<'a, P: AsRef<Path>> {
	path: P,
	resolver: &'a ContainerResolver,
	format: Format,
}

impl<'a, P: AsRef<Path>> OutputBuilder<'a, P> {
	pub fn inherit_from(mut self, input: &crate::io::Input) -> Self {
		if let Some(audio) = input.tracks.primary_audio().ok().and_then(|t| t.audio_format()) {
			self.format.inherit_audio(audio);
		}
		self
	}

	pub fn format_mut(&mut self) -> &mut Format {
		&mut self.format
	}

	pub fn format_codec(&mut self, codec: &Option<String>) -> Result<()> {
		if let Some(codec) = codec.as_ref() {
			self.format.apply_codec(codec)?;
		}
		Ok(())
	}
	pub fn build(self) -> Result<Output> {
		let path_ref = self.path.as_ref();
		let extension = utils::extension_from_path(path_ref)?;
		let path_str = path_ref.to_string_lossy();

		let file = File::create(&path_str)?;
		let muxer = self.resolver.open_muxer(&extension, file, &self.format)?;

		Ok(Output { path: path_ref.to_path_buf(), extension, format: self.format, muxer })
	}
}

pub struct Output {
	pub path: PathBuf,
	pub extension: String,
	format: Format,
	muxer: Box<dyn Muxer>,
}

impl Output {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let resolver = ContainerResolver::new();
		Self::from_resolver(path, &resolver)
	}

	pub fn builder<'a, P: AsRef<Path>>(
		path: P,
		resolver: &'a ContainerResolver,
	) -> Result<OutputBuilder<'a, P>> {
		let extension = utils::extension_from_path(path.as_ref())?;
		let container = resolver.resolver_for(&extension)?;
		let format = Format::from_container(container)?;
		Ok(OutputBuilder { path, resolver, format })
	}

	#[inline]
	pub fn from_resolver<P: AsRef<Path>>(path: P, resolver: &ContainerResolver) -> Result<Self> {
		Self::builder(path, resolver)?.build()
	}

	pub const fn format(&self) -> &Format {
		&self.format
	}

	#[inline]
	pub fn with_metadata(&mut self, metadata: impl Into<Metadata>) -> &mut Self {
		self.muxer.set_metadata(Some(metadata.into()));
		self
	}

	#[inline(always)]
	pub fn write_packet(&mut self, packet: Packet) -> Result<()> {
		self.muxer.write(packet)
	}

	#[inline(always)]
	pub fn flush(&mut self) -> Result<()> {
		self.muxer.finalize()
	}

	#[inline(always)]
	pub fn finalize(&mut self) -> Result<()> {
		self.muxer.finalize()
	}

	#[inline]
	pub fn write_all<I>(&mut self, packets: I) -> Result<()>
	where
		I: IntoIterator<Item = Packet>,
	{
		for packet in packets {
			self.write_packet(packet)?;
		}
		Ok(())
	}
}
