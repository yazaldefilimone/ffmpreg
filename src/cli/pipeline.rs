use crate::codecs::{PcmDecoder, PcmEncoder, RawVideoDecoder, RawVideoEncoder};
use crate::container::{
	AviReader, AviWriter, FlacFormat, FlacReader, FlacWriter, Mp3Reader, Mp3Writer, Mp4Reader,
	Mp4Writer, OggReader, OggWriter, WavReader, WavWriter, Y4mReader, Y4mWriter,
};
use crate::core::{Decoder, Demuxer, Encoder, Muxer, Timebase, Transform};
use crate::io::{
	BufferedWriter, IoError, IoErrorKind, IoResult, MediaRead, MediaSeek, MediaWrite, SeekFrom,
};
use crate::transform::{TransformChain, parse_transform};
use std::fs::File;
use std::path::Path;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
	Wav,
	Y4m,
	Flac,
	Mp3,
	Ogg,
	Avi,
	Mp4,
	Unknown,
}

impl MediaType {
	pub fn from_extension(path: &str) -> Self {
		let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
		match ext.as_str() {
			"wav" => MediaType::Wav,
			"y4m" => MediaType::Y4m,
			"flac" => MediaType::Flac,
			"mp3" => MediaType::Mp3,
			"ogg" | "oga" => MediaType::Ogg,
			"avi" => MediaType::Avi,
			"mp4" | "m4a" | "m4v" => MediaType::Mp4,
			_ => MediaType::Unknown,
		}
	}

	pub fn is_audio(&self) -> bool {
		matches!(self, MediaType::Wav | MediaType::Flac | MediaType::Mp3 | MediaType::Ogg)
	}

	pub fn is_video(&self) -> bool {
		matches!(self, MediaType::Y4m | MediaType::Avi | MediaType::Mp4)
	}
}

pub struct FileAdapter {
	file: File,
}

impl FileAdapter {
	pub fn open(path: &str) -> IoResult<Self> {
		let file = File::open(path)?;
		Ok(Self { file })
	}

	pub fn create(path: &str) -> IoResult<Self> {
		let file = File::create(path)?;
		Ok(Self { file })
	}
}

impl MediaRead for FileAdapter {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		use std::io::Read;
		self.file.read(buf).map_err(IoError::from)
	}
}

impl MediaWrite for FileAdapter {
	fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
		use std::io::Write;
		self.file.write(buf).map_err(IoError::from)
	}

	fn flush(&mut self) -> IoResult<()> {
		use std::io::Write;
		self.file.flush().map_err(IoError::from)
	}
}

impl MediaSeek for FileAdapter {
	fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
		use std::io::Seek;
		self.file.seek(pos.into()).map_err(IoError::from)
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

	pub fn run(&self) -> std::io::Result<()> {
		self.run_io().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
	}

	fn run_io(&self) -> IoResult<()> {
		let input_type = MediaType::from_extension(&self.input_path);
		let output_type =
			self.output_path.as_ref().map(|p| MediaType::from_extension(p)).unwrap_or(input_type);

		if self.show_mode {
			return self.run_show(input_type);
		}

		match (input_type, output_type) {
			(MediaType::Wav, MediaType::Wav) => self.run_wav_to_wav(),
			(MediaType::Wav, MediaType::Flac) => self.run_wav_to_flac(),
			(MediaType::Flac, MediaType::Wav) => self.run_flac_to_wav(),
			(MediaType::Flac, MediaType::Flac) => self.run_flac_to_flac(),
			(MediaType::Mp3, MediaType::Mp3) => self.run_mp3_passthrough(),
			(MediaType::Mp3, MediaType::Wav) => self.run_mp3_to_wav(),
			(MediaType::Ogg, MediaType::Ogg) => self.run_ogg_passthrough(),
			(MediaType::Y4m, MediaType::Y4m) => self.run_y4m_transcode(),
			(MediaType::Avi, MediaType::Avi) => self.run_avi_passthrough(),
			(MediaType::Mp4, MediaType::Mp4) => self.run_mp4_passthrough(),
			(_, _) => {
				Err(IoError::with_message(IoErrorKind::InvalidData, "unsupported format conversion"))
			}
		}
	}

	fn run_show(&self, media_type: MediaType) -> IoResult<()> {
		match media_type {
			MediaType::Wav => self.run_wav_show(),
			MediaType::Flac => self.run_flac_show(),
			MediaType::Mp3 => self.run_mp3_show(),
			MediaType::Ogg => self.run_ogg_show(),
			MediaType::Y4m => self.run_y4m_show(),
			MediaType::Avi => self.run_avi_show(),
			MediaType::Mp4 => self.run_mp4_show(),
			MediaType::Unknown => {
				Err(IoError::with_message(IoErrorKind::InvalidData, "unsupported file format"))
			}
		}
	}

	fn run_wav_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let format = reader.format();
		let mut decoder = PcmDecoder::new(format);

		println!("Format: WAV");
		println!("  Channels: {}", format.channels);
		println!("  Sample Rate: {} Hz", format.sample_rate);
		println!("  Bit Depth: {}", format.bit_depth);
		println!("\nFrames:");

		let mut frame_idx = 0u64;
		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						if let Some(audio_frame) = frame.audio() {
							println!(
								"  Frame {}: pts={}, samples={}, channels={}, rate={}",
								frame_idx,
								frame.pts,
								audio_frame.nb_samples,
								audio_frame.channels,
								audio_frame.sample_rate
							);
						} else if let Some(video_frame) = frame.video() {
							println!(
								"  Frame {}: pts={}, width={}, height={}",
								frame_idx, frame.pts, video_frame.width, video_frame.height
							);
						}
						frame_idx += 1;
						if frame_idx >= 10 {
							println!("  ... (showing first 10 frames)");
							break;
						}
					}
				}
				None => break,
			}
		}

		Ok(())
	}

	fn run_flac_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let reader = FlacReader::new(input)?;
		let format = reader.format();

		println!("Format: FLAC");
		println!("  Channels: {}", format.channels);
		println!("  Sample Rate: {} Hz", format.sample_rate);
		println!("  Bits per Sample: {}", format.bits_per_sample);
		println!("  Total Samples: {}", format.total_samples);
		println!("  Min Block Size: {}", format.min_block_size);
		println!("  Max Block Size: {}", format.max_block_size);

		Ok(())
	}

	fn run_mp3_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let reader = Mp3Reader::new(input)?;
		let format = reader.format();

		println!("Format: MP3");
		println!("  Version: {:?}", format.version);
		println!("  Layer: {:?}", format.layer);
		println!("  Channels: {}", format.channels);
		println!("  Sample Rate: {} Hz", format.sample_rate);
		println!("  Bitrate: {} kbps", format.bitrate);
		println!("  Channel Mode: {:?}", format.channel_mode);

		Ok(())
	}

	fn run_ogg_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let reader = OggReader::new(input)?;
		let format = reader.format();

		println!("Format: OGG");
		println!("  Channels: {}", format.channels);
		println!("  Sample Rate: {} Hz", format.sample_rate);
		println!("  Bitstream Serial: {}", format.bitstream_serial);

		Ok(())
	}

	fn run_y4m_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = Y4mReader::new(input)?;
		let format = reader.format();
		let mut decoder = RawVideoDecoder::new(format.clone());

		println!("Format: Y4M");
		println!("  Resolution: {}x{}", format.width, format.height);
		println!("  Framerate: {}/{}", format.framerate_num, format.framerate_den);
		println!("  Colorspace: {:?}", format.colorspace);
		println!("\nFrames:");

		let mut frame_idx = 0u64;
		loop {
			match reader.read_packet()? {
				Some(packet) => {
					if let Some(frame) = decoder.decode(packet)? {
						println!(
							"  Frame {}: pts={}, size={}x{}, fps={}/{}",
							frame_idx,
							frame.pts,
							format.width,
							format.height,
							format.framerate_num,
							format.framerate_den
						);
						frame_idx += 1;
						if frame_idx >= 10 {
							println!("  ... (showing first 10 frames)");
							break;
						}
					}
				}
				None => break,
			}
		}

		Ok(())
	}

	fn run_avi_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let reader = AviReader::new(input)?;
		let format = reader.format();

		println!("Format: AVI");
		println!("  Resolution: {}x{}", format.main_header.width, format.main_header.height);
		println!("  Total Frames: {}", format.main_header.total_frames);
		println!(
			"  Framerate: ~{:.2} fps",
			1_000_000.0 / format.main_header.microseconds_per_frame as f64
		);
		println!("  Streams: {}", format.streams.len());

		for (i, stream) in format.streams.iter().enumerate() {
			println!("  Stream {}: {:?}", i, stream.header.stream_type);
		}

		Ok(())
	}

	fn run_mp4_show(&self) -> IoResult<()> {
		let input = FileAdapter::open(&self.input_path)?;
		let reader = Mp4Reader::new(input)?;
		let format = reader.format();

		println!("Format: MP4");
		println!("  Brand: {}", String::from_utf8_lossy(&format.major_brand));
		println!("  Timescale: {}", format.timescale);
		println!("  Duration: {}", format.duration);
		println!("  Tracks: {}", format.tracks.len());

		for (i, track) in format.tracks.iter().enumerate() {
			println!("  Track {}: {:?}", i, track.track_type);
			if track.width > 0 && track.height > 0 {
				println!("    Resolution: {}x{}", track.width, track.height);
			}
			if track.sample_rate > 0 {
				println!("    Sample Rate: {}", track.sample_rate);
				println!("    Channels: {}", track.channels);
			}
		}

		Ok(())
	}

	fn run_wav_to_wav(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let format = reader.format();

		let output = FileAdapter::create(&output_path)?;
		let mut writer = WavWriter::new(output, format)?;

		let mut decoder = PcmDecoder::new(format);
		let timebase = Timebase::new(1, format.sample_rate);
		let mut encoder = PcmEncoder::new(timebase);

		let mut transform_chain = self.build_transform_chain()?;

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

	fn run_wav_to_flac(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = WavReader::new(input)?;
		let wav_format = reader.format();

		let flac_format = FlacFormat {
			sample_rate: wav_format.sample_rate,
			channels: wav_format.channels,
			bits_per_sample: wav_format.bit_depth as u8,
			..FlacFormat::default()
		};

		let output = FileAdapter::create(&output_path)?;
		let mut writer = FlacWriter::new(output, flac_format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_flac_to_wav(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = FlacReader::new(input)?;
		let flac_format = reader.format();

		let wav_format = crate::container::WavFormat {
			sample_rate: flac_format.sample_rate,
			channels: flac_format.channels,
			bit_depth: flac_format.bits_per_sample as u16,
		};

		let output = FileAdapter::create(&output_path)?;
		let mut writer = WavWriter::new(output, wav_format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_flac_to_flac(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = FlacReader::new(input)?;
		let format = reader.format().clone();

		let output = FileAdapter::create(&output_path)?;
		let mut writer = FlacWriter::new(output, format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_mp3_passthrough(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = Mp3Reader::new(input)?;

		let output = FileAdapter::create(&output_path)?;
		let mut writer = Mp3Writer::new(output)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_mp3_to_wav(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = Mp3Reader::new(input)?;
		let mp3_format = reader.format();

		let wav_format = crate::container::WavFormat {
			sample_rate: mp3_format.sample_rate,
			channels: mp3_format.channels,
			bit_depth: 16,
		};

		let output = FileAdapter::create(&output_path)?;
		let mut writer = WavWriter::new(output, wav_format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_ogg_passthrough(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = OggReader::new(input)?;
		let format = reader.format();

		let output = FileAdapter::create(&output_path)?;
		let mut writer = OggWriter::new(output, format.bitstream_serial)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_y4m_transcode(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = Y4mReader::new(input)?;
		let format = reader.format();

		let output = FileAdapter::create(&output_path)?;
		let buf_writer: BufferedWriter<FileAdapter> = BufferedWriter::new(output);
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

	fn run_avi_passthrough(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = AviReader::new(input)?;
		let format = reader.format().clone();

		let output = FileAdapter::create(&output_path)?;
		let mut writer = AviWriter::new(output, format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn run_mp4_passthrough(&self) -> IoResult<()> {
		let output_path = self.require_output()?;

		let input = FileAdapter::open(&self.input_path)?;
		let mut reader = Mp4Reader::new(input)?;
		let format = reader.format().clone();

		let output = FileAdapter::create(&output_path)?;
		let mut writer = Mp4Writer::new(output, format)?;

		loop {
			match reader.read_packet()? {
				Some(packet) => {
					writer.write_packet(packet)?;
				}
				None => break,
			}
		}

		writer.finalize()?;
		Ok(())
	}

	fn require_output(&self) -> IoResult<String> {
		self.output_path.clone().ok_or_else(|| {
			IoError::with_message(IoErrorKind::InvalidData, "output path required for transcoding")
		})
	}

	fn build_transform_chain(&self) -> IoResult<TransformChain> {
		let mut transform_chain = TransformChain::new();
		for spec in &self.transforms {
			let t = parse_transform(spec)?;
			transform_chain.add(t);
		}
		Ok(transform_chain)
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

	pub fn run(&self) -> std::io::Result<()> {
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

	fn expand_glob(&self) -> std::io::Result<Vec<String>> {
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
						eprintln!("warning: failed to read entry: {}", e);
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
