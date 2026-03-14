pub mod router;

use crate::core::Selector;
use crate::core::Track;
use crate::core::filter::FilterGraph;
use crate::core::frame::Frame;
use crate::core::packet::Packet;
use crate::core::resampler::AudioResampler;
use crate::core::resolver::CodecResolver;
use crate::core::track::{AudioFormat, Format, TrackFormat};
use crate::core::traits::{Decoder, Encoder, Resampler, Transform};
use crate::message::Result;

pub use router::Router;

pub struct Transcoder {
	decoder: Box<dyn Decoder>,
	encoder: Box<dyn Encoder>,
	resampler: Option<Box<dyn Resampler>>,
	transforms: FilterGraph,
}

impl Transcoder {
	pub fn new(decoder: Box<dyn Decoder>, encoder: Box<dyn Encoder>) -> Self {
		let transforms = FilterGraph::new();
		Self { decoder, encoder, resampler: None, transforms }
	}

	pub fn from_track(track: &Track, codecs: &CodecResolver, format: &Format) -> Result<Self> {
		let decoder = codecs.decoder_for(track)?;
		let encoder = codecs.encoder_for(track, format)?;
		let mut transcoder = Self::new(decoder, encoder);

		if let Some(input_format) = track.audio_format() {
			if let TrackFormat::Audio(output_format) = transcoder.encoder_input_format() {
				transcoder.needed_resampler(*input_format, output_format);
			}
		}

		Ok(transcoder)
	}

	pub fn with_resampler(mut self, resampler: Box<dyn Resampler>) -> Self {
		self.resampler = Some(resampler);
		self
	}

	pub fn needed_resampler(&mut self, input: AudioFormat, output: AudioFormat) {
		let resampler = AudioResampler::new(input, output);
		if resampler.needed() {
			self.resampler = Some(Box::new(resampler));
		}
	}

	pub fn add_transform(&mut self, selector: Selector, transform: Box<dyn Transform>) {
		self.transforms.add(selector, transform);
	}

	pub fn encoder_input_format(&self) -> TrackFormat {
		self.encoder.format()
	}

	pub fn transcode(
		&mut self,
		packet: Packet,
		write_packet: &mut impl FnMut(Packet) -> Result<()>,
	) -> Result<()> {
		let frames = self.decoder.decode(packet)?;
		for frame in frames {
			let frame = self.process(frame)?;
			for packet in self.encoder.encode(frame)? {
				write_packet(packet)?;
			}
		}
		Ok(())
	}

	pub fn flush(&mut self, write_packet: &mut impl FnMut(Packet) -> Result<()>) -> Result<()> {
		let frames = self.decoder.finish()?;
		for frame in frames {
			let frame = self.process(frame)?;
			for packet in self.encoder.encode(frame)? {
				write_packet(packet)?;
			}
		}
		for packet in self.encoder.finish()? {
			write_packet(packet)?;
		}
		Ok(())
	}

	fn process(&mut self, frame: Frame) -> Result<Frame> {
		let frame = self.maybe_resample(frame)?;
		let frame = self.transforms.apply(frame)?;
		Ok(frame)
	}

	fn maybe_resample(&mut self, frame: Frame) -> Result<Frame> {
		match &mut self.resampler {
			Some(resampler) => resampler.resample(frame),
			None => Ok(frame),
		}
	}
}
