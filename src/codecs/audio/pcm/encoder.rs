use crate::container::wav::format;
use crate::core::Encoder;
use crate::core::frame::{BitDepth, Channels, Frame, SampleRate};
use crate::core::packet::{Packet, PacketIter};
use crate::core::time::Time;
use crate::core::track::{AudioFormat, TrackFormat};
use crate::message::Result;

pub struct PcmEncoder {
	sample_rate: SampleRate,
	channels: Channels,
	bit_depth: BitDepth,
}

impl PcmEncoder {
	pub fn new(sample_rate: SampleRate, channels: Channels, bit_depth: BitDepth) -> Self {
		Self { sample_rate, channels, bit_depth }
	}

	#[inline(always)]
	pub fn from_format(format: &format::WavFormat) -> Self {
		Self::new(format.sample_rate, format.channels, format.bit_depth)
	}
}

impl Encoder for PcmEncoder {
	fn format(&self) -> TrackFormat {
		TrackFormat::Audio(AudioFormat {
			sample_rate: self.sample_rate,
			channels: self.channels,
			bit_depth: self.bit_depth,
		})
	}

	fn encode(&mut self, mut frame: Frame) -> Result<PacketIter> {
		let audio = frame.audio_mut().ok_or_else(|| crate::error!("no audio data"))?;
		let time = Time::new(1, self.sample_rate.value())?;
		let data = std::mem::take(&mut audio.data);
		let packet = Packet::new(data, frame.track_id, time).with_pts(frame.pts().pts);
		Ok(PacketIter::new(vec![packet]))
	}

	fn finish(&mut self) -> Result<PacketIter> {
		Ok(PacketIter::empty())
	}
}
