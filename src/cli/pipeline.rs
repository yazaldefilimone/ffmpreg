use crate::codecs::{PcmDecoder, PcmEncoder, RawVideoDecoder, RawVideoEncoder};
use crate::container::{WavReader, WavWriter, Y4mReader, Y4mWriter};
use crate::core::{Decoder, Demuxer, Encoder, Muxer, Timebase, Transform};
use crate::transform::{TransformChain, parse_transform};
use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
	Wav,
	Y4m,
	Unknown,
}

impl MediaType {
	pub fn from_extension(path: &str) -> Self {
		let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
		match ext.as_str() {
			"wav" => MediaType::Wav,
			"y4m" => MediaType::Y4m,
			_ => MediaType::Unknown,
		}
	}
}

pub struct Pipeline {
	input_path: String,
	output_path: Option<String>,
	show_mode: bool,
	transforms: Vec<String>,
}

impl Pipeline {
	pub fn new(
		input_path: String,
		output_path: Option<String>,
		show_mode: bool,
		transforms: Vec<String>,
	) -> Self {
		Self { input_path, output_path, show_mode, transforms }
	}

	pub fn run(&self) -> Result<()> {
		let media_type = MediaType::from_extension(&self.input_path);

		match media_type {
			MediaType::Wav => {
				if self.show_mode {
					self.run_wav_show()
				} else {
					self.run_wav_transcode()
				}
			}
			MediaType::Y4m => {
				if self.show_mode {
					self.run_y4m_show()
				} else {
					self.run_y4m_transcode()
				}
			}
			MediaType::Unknown => Err(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				format!("unsupported file format: {}", self.input_path),
			)),
		}
	}

	fn run_wav_show(&self) -> Result<()> {
		let input = File::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let format = reader.format();
		let mut decoder = PcmDecoder::new(format);

		let mut frame_idx = 0u64;
		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						println!(
							"Frame {}: pts={}, samples={}, channels={}, rate={}",
							frame_idx, frame.pts, frame.nb_samples, frame.channels, frame.sample_rate
						);
						frame_idx += 1;
					}
				}
				None => break,
			}
		}

		Ok(())
	}

	fn run_wav_transcode(&self) -> Result<()> {
		let output_path = self.output_path.as_ref().ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidInput, "output path required for transcoding")
		})?;

		let input = File::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let format = reader.format();

		let output = File::create(output_path)?;
		let mut writer = WavWriter::new(output, format)?;

		let mut decoder = PcmDecoder::new(format);
		let timebase = Timebase::new(1, format.sample_rate);
		let mut encoder = PcmEncoder::new(timebase);

		let mut transform_chain = TransformChain::new();
		for spec in &self.transforms {
			let t = parse_transform(spec)?;
			transform_chain.add(t);
		}

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						let processed =
							if transform_chain.is_empty() { frame } else { transform_chain.apply(frame)? };
						if let Some(pkt) = encoder.encode(processed)? {
							writer.write_packet(pkt)?;
						}
					}
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_y4m_show(&self) -> Result<()> {
		let input = File::open(&self.input_path)?;
		let mut reader = Y4mReader::new(input)?;
		let format = reader.format();
		let mut decoder = RawVideoDecoder::new(format.clone());

		let mut frame_idx = 0u64;
		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						println!(
							"Frame {}: pts={}, size={}x{}, fps={}/{}",
							frame_idx,
							frame.pts,
							format.width,
							format.height,
							format.framerate_num,
							format.framerate_den
						);
						frame_idx += 1;
					}
				}
				None => break,
			}
		}

		Ok(())
	}

	fn run_y4m_transcode(&self) -> Result<()> {
		let output_path = self.output_path.as_ref().ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidInput, "output path required for transcoding")
		})?;

		let input = File::open(&self.input_path)?;
		let mut reader = Y4mReader::new(input)?;
		let format = reader.format();

		let output = File::create(output_path)?;
		let buf_writer = BufWriter::new(output);
		let mut writer = Y4mWriter::new(buf_writer, format.clone())?;

		let timebase = Timebase::new(format.framerate_den, format.framerate_num);
		let mut decoder = RawVideoDecoder::new(format);
		let mut encoder = RawVideoEncoder::new(timebase);

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						if let Some(pkt) = encoder.encode(frame)? {
							writer.write_packet(pkt)?;
						}
					}
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}
}

pub struct BatchPipeline {
	input_pattern: String,
	output_dir: String,
	show_mode: bool,
	transforms: Vec<String>,
}

impl BatchPipeline {
	pub fn new(
		input_pattern: String,
		output_dir: String,
		show_mode: bool,
		transforms: Vec<String>,
	) -> Self {
		Self { input_pattern, output_dir, show_mode, transforms }
	}

	pub fn run(&self) -> Result<()> {
		let files = self.expand_glob()?;

		if files.is_empty() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("no files matching pattern: {}", self.input_pattern),
			));
		}

		std::fs::create_dir_all(&self.output_dir)?;

		for input_path in files {
			let file_name =
				Path::new(&input_path).file_name().and_then(|n| n.to_str()).unwrap_or("output.wav");

			let output_path =
				if self.show_mode { None } else { Some(format!("{}/{}", self.output_dir, file_name)) };

			let pipeline = Pipeline::new(
				input_path.clone(),
				output_path.clone(),
				self.show_mode,
				self.transforms.clone(),
			);

			println!("Processing: {}", input_path);
			pipeline.run()?;

			if let Some(out) = output_path {
				println!("  -> {}", out);
			}
		}

		Ok(())
	}

	fn expand_glob(&self) -> Result<Vec<String>> {
		let mut files = Vec::new();

		if self.input_pattern.contains('*') {
			let pattern = &self.input_pattern;
			for entry in glob::glob(pattern).map_err(|e| {
				std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					format!("invalid glob pattern: {}", e),
				)
			})? {
				match entry {
					Ok(path) => {
						if path.is_file() {
							files.push(path.to_string_lossy().to_string());
						}
					}
					Err(e) => {
						eprintln!("Warning: failed to read entry: {}", e);
					}
				}
			}
		} else {
			files.push(self.input_pattern.clone());
		}

		Ok(files)
	}
}

pub fn is_batch_pattern(input: &str) -> bool {
	input.contains('*')
}

pub fn is_directory(path: &str) -> bool {
	Path::new(path).is_dir()
}
