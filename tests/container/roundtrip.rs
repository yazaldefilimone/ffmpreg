use ffmpreg::codecs::{FlacEncoder, PcmDecoder, PcmEncoder};
use ffmpreg::container::{FlacFormat, FlacWriter, Mp3Writer, OggWriter, WavReader, WavWriter};
use ffmpreg::core::{Decoder, Demuxer, Encoder, Muxer, Timebase};
use ffmpreg::io::Cursor;

fn generate_sine_wave(samples: usize, frequency: f32, sample_rate: u32) -> Vec<i16> {
	let mut data = Vec::with_capacity(samples);
	for i in 0..samples {
		let t = i as f32 / sample_rate as f32;
		let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 16000.0;
		data.push(sample as i16);
	}
	data
}

fn create_mono_wav(sample_count: usize) -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 1;
	let bits_per_sample: u16 = 16;

	let samples = generate_sine_wave(sample_count, 440.0, sample_rate);
	let data_size = samples.len() * 2;
	let file_size = 36 + data_size;

	let mut wav = Vec::new();

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&(file_size as u32).to_le_bytes());
	wav.extend_from_slice(b"WAVE");

	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&16u32.to_le_bytes());
	wav.extend_from_slice(&1u16.to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	let block_align = channels * bits_per_sample / 8;
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());

	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&(data_size as u32).to_le_bytes());

	for sample in samples {
		wav.extend_from_slice(&sample.to_le_bytes());
	}

	wav
}

fn create_stereo_wav(sample_count: usize) -> Vec<u8> {
	let sample_rate: u32 = 44100;
	let channels: u16 = 2;
	let bits_per_sample: u16 = 16;

	let left = generate_sine_wave(sample_count, 440.0, sample_rate);
	let right = generate_sine_wave(sample_count, 880.0, sample_rate);

	let mut samples = Vec::with_capacity(sample_count * 2);
	for (l, r) in left.iter().zip(right.iter()) {
		samples.push(*l);
		samples.push(*r);
	}

	let data_size = samples.len() * 2;
	let file_size = 36 + data_size;

	let mut wav = Vec::new();

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&(file_size as u32).to_le_bytes());
	wav.extend_from_slice(b"WAVE");

	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&16u32.to_le_bytes());
	wav.extend_from_slice(&1u16.to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	let block_align = channels * bits_per_sample / 8;
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());

	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&(data_size as u32).to_le_bytes());

	for sample in samples {
		wav.extend_from_slice(&sample.to_le_bytes());
	}

	wav
}

#[test]
fn test_wav_roundtrip_mono_demux_mux() {
	let original_wav = create_mono_wav(512);
	let original_len = original_wav.len();

	let cursor = Cursor::new(original_wav.clone());
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let mut packets = Vec::new();
	while let Some(packet) = reader.read_packet().unwrap() {
		packets.push(packet);
	}

	assert!(!packets.is_empty(), "no packets read");

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut total_bytes = 0;
	for packet in packets {
		total_bytes += packet.size();
		writer.write_packet(packet).unwrap();
	}
	writer.finalize().unwrap();

	assert!(total_bytes > 0, "no bytes written");
	assert_eq!(total_bytes as usize, original_len - 44, "data size mismatch");
}

#[test]
fn test_wav_roundtrip_stereo_demux_mux() {
	let original_wav = create_stereo_wav(256);
	let cursor = Cursor::new(original_wav.clone());
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.channels, 2, "expected stereo format");

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut packet_count = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		packet_count += 1;
		writer.write_packet(packet).unwrap();
	}

	writer.finalize().unwrap();
	assert!(packet_count > 0, "no packets processed");
}

#[test]
fn test_wav_roundtrip_full_pipeline() {
	let original_wav = create_mono_wav(512);
	let cursor = Cursor::new(original_wav);
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut frames_processed = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			frames_processed += 1;
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
	assert!(frames_processed > 0, "no frames decoded");
}

#[test]
fn test_flac_roundtrip_mono_demux_mux() {
	let wav_source = create_mono_wav(512);
	let wav_cursor = Cursor::new(wav_source);
	let mut wav_reader = WavReader::new(wav_cursor).unwrap();
	let wav_format = wav_reader.format();

	let mut decoder = PcmDecoder::new(wav_format);
	let mut pcm_packets = Vec::new();

	while let Some(packet) = wav_reader.read_packet().unwrap() {
		pcm_packets.push(packet);
	}

	let flac_format = FlacFormat {
		min_block_size: 4096,
		max_block_size: 4096,
		min_frame_size: 0,
		max_frame_size: 0,
		sample_rate: wav_format.sample_rate,
		channels: wav_format.channels,
		bits_per_sample: 16,
		total_samples: 512,
		md5_signature: [0u8; 16],
	};

	let output_buffer = Cursor::new(Vec::new());
	let mut flac_writer = FlacWriter::new(output_buffer, flac_format).unwrap();

	let mut packet_count = 0;
	let mut flac_encoder = FlacEncoder::new(wav_format.sample_rate, wav_format.channels, 16, 4096);

	for pcm_packet in pcm_packets {
		if let Some(frame) = decoder.decode(pcm_packet).unwrap() {
			if let Some(flac_pkt) = flac_encoder.encode(frame).unwrap() {
				packet_count += 1;
				flac_writer.write_packet(flac_pkt).unwrap();
			}
		}
	}

	flac_writer.finalize().unwrap();
	assert!(packet_count > 0, "no FLAC packets written");
}

#[test]
fn test_flac_roundtrip_full_pipeline() {
	let wav_source = create_mono_wav(256);
	let wav_cursor = Cursor::new(wav_source);
	let mut wav_reader = WavReader::new(wav_cursor).unwrap();
	let wav_format = wav_reader.format();

	let flac_format = FlacFormat {
		min_block_size: 4096,
		max_block_size: 4096,
		min_frame_size: 0,
		max_frame_size: 0,
		sample_rate: wav_format.sample_rate,
		channels: wav_format.channels,
		bits_per_sample: 16,
		total_samples: 256,
		md5_signature: [0u8; 16],
	};

	let output_buffer = Cursor::new(Vec::new());
	let mut flac_writer = FlacWriter::new(output_buffer, flac_format.clone()).unwrap();

	let mut pcm_decoder = PcmDecoder::new(wav_format);
	let mut flac_encoder = FlacEncoder::new(wav_format.sample_rate, wav_format.channels, 16, 4096);

	let mut frames_written = 0;
	while let Some(packet) = wav_reader.read_packet().unwrap() {
		if let Some(frame) = pcm_decoder.decode(packet).unwrap() {
			if let Some(flac_pkt) = flac_encoder.encode(frame).unwrap() {
				flac_writer.write_packet(flac_pkt).unwrap();
				frames_written += 1;
			}
		}
	}

	flac_writer.finalize().unwrap();
	assert!(frames_written > 0, "no FLAC frames written");
}

#[test]
fn test_mp3_roundtrip_read_write_identity() {
	let wav_source = create_mono_wav(512);
	let wav_cursor = Cursor::new(wav_source);
	let mut wav_reader = WavReader::new(wav_cursor).unwrap();
	let wav_format = wav_reader.format();

	let mut pcm_decoder = PcmDecoder::new(wav_format);
	let mut frame_data_samples = Vec::new();

	while let Some(packet) = wav_reader.read_packet().unwrap() {
		if let Some(frame) = pcm_decoder.decode(packet).unwrap() {
			if let Some(audio) = frame.audio() {
				frame_data_samples.push(audio.data.clone());
			}
		}
	}

	assert!(!frame_data_samples.is_empty(), "no audio frames decoded from WAV");

	let output_buffer = Cursor::new(Vec::new());
	let mut mp3_writer = Mp3Writer::new(output_buffer).unwrap();

	let mut packets_written = 0;
	let frame_count = frame_data_samples.len();
	for frame_data in frame_data_samples {
		let dummy_packet =
			ffmpreg::core::Packet::new(frame_data, 0, Timebase::new(1, wav_format.sample_rate));
		mp3_writer.write_packet(dummy_packet).unwrap();
		packets_written += 1;
	}

	mp3_writer.finalize().unwrap();

	assert!(packets_written > 0, "no packets written to MP3, processed {} audio frames", frame_count);
}

#[test]
fn test_ogg_roundtrip_vorbis_format() {
	let wav_source = create_mono_wav(512);
	let wav_cursor = Cursor::new(wav_source);
	let mut wav_reader = WavReader::new(wav_cursor).unwrap();
	let _wav_format = wav_reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut ogg_writer = OggWriter::new(output_buffer, 42).unwrap();

	let mut packet_count = 0;
	while let Some(packet) = wav_reader.read_packet().unwrap() {
		packet_count += 1;
		ogg_writer.write_packet(packet).unwrap();
	}

	ogg_writer.finalize().unwrap();

	assert!(packet_count > 0, "no packets written to OGG");
}

#[test]
fn test_multi_container_batch() {
	let test_samples = 256;
	let wav_source = create_mono_wav(test_samples);

	for iteration in 0..3 {
		let cursor = Cursor::new(wav_source.clone());
		let mut reader = WavReader::new(cursor).unwrap();
		let format = reader.format();

		let output = Cursor::new(Vec::new());
		let mut writer = WavWriter::new(output, format).unwrap();

		let mut pcm_decoder = PcmDecoder::new(format);
		let mut pcm_encoder = PcmEncoder::new(Timebase::new(1, format.sample_rate));

		let mut frame_count = 0;
		while let Some(packet) = reader.read_packet().unwrap() {
			if let Some(frame) = pcm_decoder.decode(packet).unwrap() {
				frame_count += 1;
				if let Some(pkt) = pcm_encoder.encode(frame).unwrap() {
					writer.write_packet(pkt).unwrap();
				}
			}
		}

		writer.finalize().unwrap();
		assert!(frame_count > 0, "iteration {}: no frames processed", iteration);
	}
}

#[test]
fn test_wav_stereo_roundtrip_full_pipeline() {
	let original_wav = create_stereo_wav(512);
	let cursor = Cursor::new(original_wav);
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.channels, 2);

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut total_samples = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			if let Some(audio) = frame.audio() {
				total_samples += audio.nb_samples;
			}
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
	assert_eq!(total_samples, 512, "expected 512 total samples");
}

#[test]
fn test_wav_pts_preservation_roundtrip() {
	let original_wav = create_mono_wav(512);
	let cursor = Cursor::new(original_wav);
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut last_pts = 0i64;
	let mut pts_errors = 0;

	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			let frame_pts = frame.pts;
			if frame_pts < last_pts {
				pts_errors += 1;
			}
			last_pts = frame_pts;

			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
	assert_eq!(pts_errors, 0, "PTS ordering violated");
	assert!(last_pts >= 0, "final PTS should be non-negative");
}

#[test]
fn test_flac_stereo_roundtrip() {
	let wav_source = create_stereo_wav(256);
	let wav_cursor = Cursor::new(wav_source);
	let mut wav_reader = WavReader::new(wav_cursor).unwrap();
	let wav_format = wav_reader.format();

	assert_eq!(wav_format.channels, 2);

	let flac_format = FlacFormat {
		min_block_size: 4096,
		max_block_size: 4096,
		min_frame_size: 0,
		max_frame_size: 0,
		sample_rate: wav_format.sample_rate,
		channels: wav_format.channels,
		bits_per_sample: 16,
		total_samples: 256,
		md5_signature: [0u8; 16],
	};

	let output_buffer = Cursor::new(Vec::new());
	let mut flac_writer = FlacWriter::new(output_buffer, flac_format).unwrap();

	let mut pcm_decoder = PcmDecoder::new(wav_format);
	let mut flac_encoder = FlacEncoder::new(wav_format.sample_rate, wav_format.channels, 16, 4096);

	let mut frames_written = 0;
	while let Some(packet) = wav_reader.read_packet().unwrap() {
		if let Some(frame) = pcm_decoder.decode(packet).unwrap() {
			if let Some(flac_pkt) = flac_encoder.encode(frame).unwrap() {
				flac_writer.write_packet(flac_pkt).unwrap();
				frames_written += 1;
			}
		}
	}

	flac_writer.finalize().unwrap();
	assert!(frames_written > 0);
}

#[test]
fn test_container_format_preservation() {
	let original_wav = create_mono_wav(512);
	let cursor = Cursor::new(original_wav);
	let reader = WavReader::new(cursor).unwrap();
	let original_format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let _writer = WavWriter::new(output_buffer, original_format).unwrap();

	assert_eq!(original_format.sample_rate, 44100);
	assert_eq!(original_format.channels, 1);
	assert_eq!(original_format.bit_depth, 16);
}

#[test]
fn test_large_sample_roundtrip() {
	let large_sample_count = 4096;
	let original_wav = create_mono_wav(large_sample_count);
	let cursor = Cursor::new(original_wav);
	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut total_samples = 0;
	while let Some(packet) = reader.read_packet().unwrap() {
		if let Some(frame) = decoder.decode(packet).unwrap() {
			if let Some(audio) = frame.audio() {
				total_samples += audio.nb_samples;
			}
			if let Some(pkt) = encoder.encode(frame).unwrap() {
				writer.write_packet(pkt).unwrap();
			}
		}
	}

	writer.finalize().unwrap();
	assert_eq!(total_samples, large_sample_count);
}
