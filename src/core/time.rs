use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeBase {
	pub num: u32,
	pub den: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
	ticks: u64,
	base: TimeBase,
}

impl TimeBase {
	pub const fn new(num: u32, den: u32) -> Self {
		Self { num, den }
	}
}

impl Time {
	pub const fn new(ticks: u64, base: TimeBase) -> Self {
		Self { ticks, base }
	}

	pub const fn zero(base: TimeBase) -> Self {
		Self { ticks: 0, base }
	}

	pub const fn from_seconds(seconds: f64, base: TimeBase) -> Self {
		let ticks = (seconds * (base.den as f64) / (base.num as f64)).round() as u64;
		Self { ticks, base }
	}

	pub const fn as_seconds(&self) -> f64 {
		self.ticks as f64 * (self.base.num as f64) / (self.base.den as f64)
	}

	pub fn checked_add(self, other: Time) -> Option<Self> {
		if self.base != other.base {
			return None;
		}
		self.ticks.checked_add(other.ticks).map(|ticks| Self { ticks, base: self.base })
	}

	pub fn checked_sub(self, other: Time) -> Option<Self> {
		if self.base != other.base {
			return None;
		}
		self.ticks.checked_sub(other.ticks).map(|ticks| Self { ticks, base: self.base })
	}

	pub fn unpack(&self) -> (u64, u64, u64, u64) {
		let ticks = self.ticks;
		let base = self.base;

		let total_ms = ticks * base.num as u64 * 1000 / base.den as u64;

		let total_seconds = total_ms / 1000;
		let ms = total_ms % 1000;

		let hour = total_seconds / 3600;
		let minutes = (total_seconds % 3600) / 60;
		let seconds = total_seconds % 60;

		(hour, minutes, seconds, ms)
	}
}

impl fmt::Display for Time {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (hour, minutes, seconds, _) = self.unpack();
		write!(f, "{:02}:{:02}:{:02}", hour, minutes, seconds)
	}
}
