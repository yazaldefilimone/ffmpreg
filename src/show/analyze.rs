use crate::codecs::{PcmDecoder, RawVideoDecoder};
use crate::container::{
	AviReader, FlacReader, Mp4Reader, WavFormat, WavReader, Y4mFormat, Y4mReader,
};
use crate::core::{Decoder, Demuxer};
use crate::io::{IoResult, MediaSeek, SeekFrom};

use super::format::bytes_to_hex;
use super::types::{
	AudioStreamInfo, FileInfo, FrameInfo, MediaInfo, ShowOptions, StreamInfo, VideoStreamInfo,
};

pub fn analyze_wav<R>(reader: R, path: &str, opts: &ShowOptions) -> IoResult<MediaInfo>
where
	R: crate::io::MediaRead + MediaSeek,
{
	let file_size = measure_file_size(reader)?;
	let input = open_file(path)?;
	let mut wav_reader = WavReader::new(input)?;
	let format = wav_reader.format();

	let duration = calculate_wav_duration(&format, file_size);
	let stream = build_audio_stream(&format);
	let frames = collect_wav_frames(&mut wav_reader, &format, opts)?;

	let file_info = FileInfo { path: path.to_string(), duration, size: file_size };

	Ok(MediaInfo { file: file_info, streams: vec![stream], frames })
}

pub fn analyze_y4m<R>(reader: R, path: &str, opts: &ShowOptions) -> IoResult<MediaInfo>
where
	R: crate::io::MediaRead + MediaSeek,
{
	let file_size = measure_file_size(reader)?;
	let input = open_file(path)?;
	let mut y4m_reader = Y4mReader::new(input)?;
	let format = y4m_reader.format();

	let duration = calculate_y4m_duration(&format, file_size);
	let stream = build_video_stream(&format);
	let frames = collect_y4m_frames(&mut y4m_reader, &format, opts)?;

	let file_info = FileInfo { path: path.to_string(), duration, size: file_size };

	Ok(MediaInfo { file: file_info, streams: vec![stream], frames })
}

fn measure_file_size<R: MediaSeek>(mut reader: R) -> IoResult<u64> {
	let size = reader.seek(SeekFrom::End(0))?;
	reader.seek(SeekFrom::Start(0))?;
	Ok(size)
}

fn open_file(path: &str) -> IoResult<crate::cli::pipeline::FileAdapter> {
	crate::cli::pipeline::FileAdapter::open(path)
}

fn calculate_wav_duration(format: &WavFormat, file_size: u64) -> f64 {
	let header_size = 44u64;
	let data_size = file_size.saturating_sub(header_size);
	let bytes_per_second = format.sample_rate as u64 * format.bytes_per_frame() as u64;

	if bytes_per_second == 0 {
		return 0.0;
	}

	data_size as f64 / bytes_per_second as f64
}

fn calculate_y4m_duration(format: &Y4mFormat, file_size: u64) -> f64 {
	let header_approx = 100u64;
	let frame_header = 6u64;
	let frame_size = format.frame_size() as u64 + frame_header;
	let data_size = file_size.saturating_sub(header_approx);
	let frame_count = data_size / frame_size;

	let fps = format.framerate_num as f64 / format.framerate_den as f64;

	if fps <= 0.0 {
		return 0.0;
	}

	frame_count as f64 / fps
}

fn build_audio_stream(format: &WavFormat) -> StreamInfo {
	let info = AudioStreamInfo {
		index: 0,
		codec: "pcm_s16le".to_string(),
		sample_rate: format.sample_rate,
		channels: format.channels,
		bit_depth: format.bit_depth,
	};

	StreamInfo::Audio(info)
}

fn build_video_stream(format: &Y4mFormat) -> StreamInfo {
	let pix_fmt = resolve_pixel_format(format);
	let frame_rate = format!("{}/{}", format.framerate_num, format.framerate_den);
	let field_order = resolve_field_order(format);
	let aspect_ratio = format.aspect_ratio.map(|a| a.to_string());
	let display_aspect = calculate_display_aspect(format);

	let info = VideoStreamInfo {
		index: 0,
		codec: format!("rawvideo ({})", pix_fmt),
		pix_fmt: pix_fmt.to_string(),
		width: format.width,
		height: format.height,
		frame_rate,
		aspect_ratio,
		display_aspect,
		field_order: field_order.to_string(),
	};

	StreamInfo::Video(info)
}

fn resolve_pixel_format(format: &Y4mFormat) -> &'static str {
	let colorspace = match format.colorspace {
		Some(c) => c,
		None => return "yuv420p",
	};

	use crate::container::y4m::Colorspace;

	match colorspace {
		Colorspace::C420 | Colorspace::C420jpeg | Colorspace::C420paldv | Colorspace::C420mpeg2 => {
			"yuv420p"
		}
		Colorspace::C422 => "yuv422p",
		Colorspace::C444 => "yuv444p",
		Colorspace::Mono => "gray",
	}
}

fn resolve_field_order(format: &Y4mFormat) -> &'static str {
	use crate::container::y4m::Interlacing;

	match format.interlacing {
		Interlacing::Progressive => "progressive",
		Interlacing::TopFieldFirst => "top field first",
		Interlacing::BottomFieldFirst => "bottom field first",
		Interlacing::Mixed => "mixed",
	}
}

fn calculate_display_aspect(format: &Y4mFormat) -> Option<String> {
	let aspect = format.aspect_ratio?;
	let dar_num = format.width * aspect.num;
	let dar_den = format.height * aspect.den;

	Some(format!("{}:{}", dar_num, dar_den))
}

fn collect_wav_frames<R: crate::io::MediaRead>(
	reader: &mut WavReader<R>,
	format: &WavFormat,
	opts: &ShowOptions,
) -> IoResult<Vec<FrameInfo>> {
	let mut frames = Vec::new();
	let mut decoder = PcmDecoder::new(*format);
	let limit = opts.frame_limit as u64;
	let mut frame_idx = 0u64;

	loop {
		let reached_limit = frame_idx >= limit;

		if reached_limit {
			break;
		}

		let packet = reader.read_packet()?;

		let Some(pkt) = packet else {
			break;
		};

		// store always a good preview (256 bytes) for xxd mode
		let hex_preview_limit = 256.max(opts.hex_limit);
		let hex = bytes_to_hex(&pkt.data, hex_preview_limit);
		let decoded = decoder.decode(pkt.clone())?;

		if decoded.is_none() {
			continue;
		}

		let info =
			FrameInfo { index: frame_idx, pts: pkt.pts, keyframe: true, size: pkt.data.len(), hex };

		frames.push(info);
		frame_idx += 1;
	}

	Ok(frames)
}

fn collect_y4m_frames<R: crate::io::MediaRead>(
	reader: &mut Y4mReader<R>,
	format: &Y4mFormat,
	opts: &ShowOptions,
) -> IoResult<Vec<FrameInfo>> {
	let mut frames = Vec::new();
	let mut decoder = RawVideoDecoder::new(format.clone());
	let limit = opts.frame_limit as u64;
	let mut frame_idx = 0u64;
	let hex_preview_limit = 256.max(opts.hex_limit);

	loop {
		let reached_limit = frame_idx >= limit;

		if reached_limit {
			break;
		}

		let packet = reader.read_packet()?;

		let Some(pkt) = packet else {
			break;
		};

		let hex = bytes_to_hex(&pkt.data, hex_preview_limit);
		let decoded = decoder.decode(pkt.clone())?;

		if decoded.is_none() {
			continue;
		}

		let info =
			FrameInfo { index: frame_idx, pts: pkt.pts, keyframe: true, size: pkt.data.len(), hex };

		frames.push(info);
		frame_idx += 1;
	}

	Ok(frames)
}

pub fn analyze_flac<R>(reader: R, path: &str, _opts: &ShowOptions) -> IoResult<MediaInfo>
where
	R: crate::io::MediaRead + MediaSeek,
{
	let file_size = measure_file_size(reader)?;
	let input = open_file(path)?;
	let flac_reader = FlacReader::new(input)?;
	let format = flac_reader.format();

	let duration = if format.sample_rate > 0 {
		format.total_samples as f64 / format.sample_rate as f64
	} else {
		0.0
	};

	let stream = StreamInfo::Audio(AudioStreamInfo {
		index: 0,
		codec: "flac".to_string(),
		sample_rate: format.sample_rate,
		channels: format.channels,
		bit_depth: format.bits_per_sample as u16,
	});

	let file_info = FileInfo { path: path.to_string(), duration, size: file_size };
	Ok(MediaInfo { file: file_info, streams: vec![stream], frames: Vec::new() })
}

pub fn analyze_avi<R>(reader: R, path: &str, _opts: &ShowOptions) -> IoResult<MediaInfo>
where
	R: crate::io::MediaRead + MediaSeek,
{
	let file_size = measure_file_size(reader)?;
	let input = open_file(path)?;
	let avi_reader = AviReader::new(input)?;
	let format = avi_reader.format();

	let fps = if format.main_header.microseconds_per_frame > 0 {
		1_000_000.0 / format.main_header.microseconds_per_frame as f64
	} else {
		30.0
	};
	let duration = format.main_header.total_frames as f64 / fps;

	let mut streams = Vec::new();
	for (i, stream) in format.streams.iter().enumerate() {
		match stream.header.stream_type {
			crate::container::avi::StreamType::Video => {
				if let Some(ref vf) = stream.video_format {
					streams.push(StreamInfo::Video(VideoStreamInfo {
						index: i,
						codec: String::from_utf8_lossy(&vf.compression).trim().to_string(),
						pix_fmt: format!("{}bpp", vf.bit_count),
						width: vf.width.unsigned_abs(),
						height: vf.height.unsigned_abs(),
						frame_rate: format!("{:.2}", fps),
						aspect_ratio: None,
						display_aspect: None,
						field_order: "progressive".to_string(),
					}));
				}
			}
			crate::container::avi::StreamType::Audio => {
				if let Some(ref af) = stream.audio_format {
					streams.push(StreamInfo::Audio(AudioStreamInfo {
						index: i,
						codec: format!("pcm (tag={})", af.format_tag),
						sample_rate: af.samples_per_sec,
						channels: af.channels as u8,
						bit_depth: af.bits_per_sample,
					}));
				}
			}
			_ => {}
		}
	}

	let file_info = FileInfo { path: path.to_string(), duration, size: file_size };
	Ok(MediaInfo { file: file_info, streams, frames: Vec::new() })
}

pub fn analyze_mp4<R>(reader: R, path: &str, _opts: &ShowOptions) -> IoResult<MediaInfo>
where
	R: crate::io::MediaRead + MediaSeek,
{
	let file_size = measure_file_size(reader)?;
	let input = open_file(path)?;
	let mp4_reader = Mp4Reader::new(input)?;
	let format = mp4_reader.format();

	let duration =
		if format.timescale > 0 { format.duration as f64 / format.timescale as f64 } else { 0.0 };

	let mut streams = Vec::new();
	for (i, track) in format.tracks.iter().enumerate() {
		match track.track_type {
			crate::container::mp4::TrackType::Video => {
				let fps = if track.timescale > 0 && track.duration > 0 {
					let sample_count = track.sample_sizes.len() as f64;
					sample_count * track.timescale as f64 / track.duration as f64
				} else {
					30.0
				};
				streams.push(StreamInfo::Video(VideoStreamInfo {
					index: i,
					codec: "h264".to_string(),
					pix_fmt: "yuv420p".to_string(),
					width: track.width,
					height: track.height,
					frame_rate: format!("{:.2}", fps),
					aspect_ratio: None,
					display_aspect: None,
					field_order: "progressive".to_string(),
				}));
			}
			crate::container::mp4::TrackType::Audio => {
				streams.push(StreamInfo::Audio(AudioStreamInfo {
					index: i,
					codec: "aac".to_string(),
					sample_rate: track.sample_rate,
					channels: track.channels as u8,
					bit_depth: 16,
				}));
			}
			_ => {}
		}
	}

	let file_info = FileInfo { path: path.to_string(), duration, size: file_size };
	Ok(MediaInfo { file: file_info, streams, frames: Vec::new() })
}
