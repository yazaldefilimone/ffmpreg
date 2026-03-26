#[derive(Debug)]
pub enum Message {
	Io(std::io::Error),
	Other(&'static str),
	Container(&'static str),
	Codec(&'static str),
}

pub type Result<T> = std::result::Result<T, Message>;

impl From<std::io::Error> for Message {
	fn from(err: std::io::Error) -> Self {
		Message::Io(err)
	}
}
impl From<&'static str> for Message {
	fn from(err: &'static str) -> Self {
		Message::Other(err)
	}
}
