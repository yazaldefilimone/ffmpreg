mod analyze;
mod format;
mod human;
mod json;
mod types;

pub use types::{MediaInfo, ShowOptions};

use crate::cli::pipeline::{FileAdapter, MediaType};
use crate::io::IoResult;

pub struct Show {
	input_path: String,
	opts: ShowOptions,
}

impl Show {
	pub fn new(input_path: String, opts: ShowOptions) -> Self {
		Self { input_path, opts }
	}

	pub fn run(&self) -> std::io::Result<()> {
		self.run_io().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
	}

	fn run_io(&self) -> IoResult<()> {
		let info = self.analyze()?;

		self.render(&info);

		Ok(())
	}

	fn analyze(&self) -> IoResult<MediaInfo> {
		let media_type = MediaType::from_extension(&self.input_path);
		let input = FileAdapter::open(&self.input_path)?;

		match media_type {
			MediaType::Wav => analyze::analyze_wav(input, &self.input_path, &self.opts),
			MediaType::Y4m => analyze::analyze_y4m(input, &self.input_path, &self.opts),
			MediaType::Flac => analyze::analyze_flac(input, &self.input_path, &self.opts),
			MediaType::Avi => analyze::analyze_avi(input, &self.input_path, &self.opts),
			MediaType::Mp4 => analyze::analyze_mp4(input, &self.input_path, &self.opts),
			MediaType::Unknown => Err(crate::io::IoError::invalid_data("unsupported file format")),
		}
	}

	fn render(&self, info: &MediaInfo) {
		if self.opts.json {
			json::render(info);
			return;
		}

		human::render(info, &self.opts);
	}
}
