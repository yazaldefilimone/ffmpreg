use crate::container::OggFormat;
use crate::core::{Decoder, Frame, FrameAudio, Packet};
use crate::io::{IoError, IoResult};
use lewton::inside_ogg::OggStreamReader;
use std::io::Cursor;

pub struct VorbisDecoder {
	format: OggFormat,
}

impl VorbisDecoder {
	pub fn new(format: OggFormat) -> Self {
		Self { format }
	}

	pub fn from_ogg_data(data: &[u8]) -> IoResult<Self> {
		let cursor = Cursor::new(data.to_vec());
		match OggStreamReader::new(cursor) {
			Ok(reader) => {
				let format = OggFormat {
					sample_rate: reader.ident_hdr.audio_sample_rate,
					channels: reader.ident_hdr.audio_channels,
					bitstream_serial: 0,
				};
				Ok(Self { format })
			}
			Err(_) => Err(IoError::invalid_data("failed to parse Vorbis stream")),
		}
	}

	pub fn sample_rate(&self) -> u32 {
		self.format.sample_rate
	}

	pub fn channels(&self) -> u8 {
		self.format.channels
	}
}

impl Decoder for VorbisDecoder {
	fn decode(&mut self, packet: Packet) -> IoResult<Option<Frame>> {
		let cursor = Cursor::new(packet.data);

		let mut reader = match OggStreamReader::new(cursor) {
			Ok(r) => r,
			Err(_) => return Ok(None),
		};

		let sample_rate = reader.ident_hdr.audio_sample_rate;
		let channels = reader.ident_hdr.audio_channels;

		let mut all_samples: Vec<i16> = Vec::new();

		while let Ok(Some(samples)) = reader.read_dec_packet_itl() {
			all_samples.extend(samples);
		}

		if all_samples.is_empty() {
			return Ok(None);
		}

		let mut output = Vec::with_capacity(all_samples.len() * 2);
		for sample in &all_samples {
			output.extend_from_slice(&sample.to_le_bytes());
		}

		let nb_samples = all_samples.len() / channels.max(1) as usize;
		let audio = FrameAudio::new(output, sample_rate, channels).with_nb_samples(nb_samples);
		let frame = Frame::new_audio(audio, packet.timebase, packet.stream_index).with_pts(packet.pts);

		Ok(Some(frame))
	}

	fn flush(&mut self) -> IoResult<Option<Frame>> {
		Ok(None)
	}
}
