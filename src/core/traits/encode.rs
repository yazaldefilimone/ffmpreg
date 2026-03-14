use crate::core::frame::Frame;
use crate::core::packet::PacketIter;
use crate::core::track::TrackFormat;
use crate::message::Result;

pub trait Encoder {
	fn format(&self) -> TrackFormat;
	fn encode(&mut self, frame: Frame) -> Result<PacketIter>;
	fn finish(&mut self) -> Result<PacketIter>;
}
