mod common;

use ffmpreg::codecs::{PcmDecoder, PcmEncoder, RawVideoDecoder, RawVideoEncoder};
use ffmpreg::container::{WavFormat, WavReader, WavWriter, Y4mReader, Y4mWriter};
use ffmpreg::core::{Decoder, Demuxer, Encoder, Muxer, Timebase, Transform};
use ffmpreg::io::{
	BufferedReader, BufferedWriter, Cursor, MediaRead, MediaSeek, MediaWrite, ReadPrimitives,
	SeekFrom, WritePrimitives,
};
use ffmpreg::transform::{Gain, Normalize, TransformChain};

#[test]
fn test_cursor_read_write_roundtrip() {
	let original = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
	let mut cursor = Cursor::new(original.clone());

	let mut buf = [0u8; 4];
	let n = cursor.read(&mut buf).unwrap();
	assert_eq!(n, 4);
	assert_eq!(&buf, &[1, 2, 3, 4]);

	let n = cursor.read(&mut buf).unwrap();
	assert_eq!(n, 4);
	assert_eq!(&buf, &[5, 6, 7, 8]);

	let n = cursor.read(&mut buf).unwrap();
	assert_eq!(n, 0);
}

#[test]
fn test_cursor_write_vec() {
	let mut cursor = Cursor::new(Vec::new());

	cursor.write_all(&[1, 2, 3, 4]).unwrap();
	cursor.write_all(&[5, 6, 7, 8]).unwrap();

	let data = cursor.into_inner();
	assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_cursor_seek() {
	let data = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
	let mut cursor = Cursor::new(data);

	cursor.seek(SeekFrom::Start(4)).unwrap();
	assert_eq!(cursor.position(), 4);

	let byte = cursor.read_u8().unwrap();
	assert_eq!(byte, 4);

	cursor.seek(SeekFrom::Current(-2)).unwrap();
	let byte = cursor.read_u8().unwrap();
	assert_eq!(byte, 3);

	cursor.seek(SeekFrom::End(-1)).unwrap();
	let byte = cursor.read_u8().unwrap();
	assert_eq!(byte, 7);
}

#[test]
fn test_buffered_reader() {
	let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
	let cursor = Cursor::new(data);
	let mut buffered: BufferedReader<_, 4> = BufferedReader::new(cursor);

	let mut buf = [0u8; 2];
	buffered.read_exact(&mut buf).unwrap();
	assert_eq!(buf, [1, 2]);

	buffered.read_exact(&mut buf).unwrap();
	assert_eq!(buf, [3, 4]);

	buffered.read_exact(&mut buf).unwrap();
	assert_eq!(buf, [5, 6]);
}

#[test]
fn test_buffered_writer() {
	let output = Vec::new();
	let mut buffered: BufferedWriter<_, 8> = BufferedWriter::new(output);

	buffered.write_all(&[1, 2, 3, 4]).unwrap();
	buffered.write_all(&[5, 6, 7, 8]).unwrap();
	buffered.flush().unwrap();

	let data = buffered.into_inner();
	assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_wav_roundtrip_with_io_abstractions() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut packet_count = 0;
	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				packet_count += 1;
				if let Some(frame) = decoder.decode(packet).unwrap() {
					if let Some(pkt) = encoder.encode(frame).unwrap() {
						writer.write_packet(pkt).unwrap();
					}
				}
			}
			None => break,
		}
	}

	assert!(packet_count > 0);
	writer.finalize().unwrap();
}

#[test]
fn test_wav_with_gain_transform_io() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);
	let mut gain = Gain::new(2.0);

	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				if let Some(frame) = decoder.decode(packet).unwrap() {
					let processed = gain.apply(frame).unwrap();
					if let Some(pkt) = encoder.encode(processed).unwrap() {
						writer.write_packet(pkt).unwrap();
					}
				}
			}
			None => break,
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_wav_with_transform_chain_io() {
	let wav_data = common::create_test_wav_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	let mut chain = TransformChain::new();
	chain.add(Box::new(Gain::new(0.5)));
	chain.add(Box::new(Normalize::new(0.9)));

	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				if let Some(frame) = decoder.decode(packet).unwrap() {
					let processed = chain.apply(frame).unwrap();
					if let Some(pkt) = encoder.encode(processed).unwrap() {
						writer.write_packet(pkt).unwrap();
					}
				}
			}
			None => break,
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_y4m_roundtrip_with_io_abstractions() {
	let y4m_data = common::create_test_y4m_data();
	let cursor = Cursor::new(y4m_data);

	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer: BufferedWriter<Cursor<Vec<u8>>> = BufferedWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format.clone()).unwrap();

	let timebase = Timebase::new(format.framerate_den, format.framerate_num);
	let mut decoder = RawVideoDecoder::new(format);
	let mut encoder = RawVideoEncoder::new(timebase);

	let mut frame_count = 0;
	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				if let Some(frame) = decoder.decode(packet).unwrap() {
					if let Some(pkt) = encoder.encode(frame).unwrap() {
						writer.write_packet(pkt).unwrap();
						frame_count += 1;
					}
				}
			}
			None => break,
		}
	}

	writer.finalize().unwrap();
	assert_eq!(frame_count, 3);
}

#[test]
fn test_stereo_wav_io_roundtrip() {
	let wav_data = common::create_test_wav_stereo_data();
	let cursor = Cursor::new(wav_data);

	let mut reader = WavReader::new(cursor).unwrap();
	let format = reader.format();

	assert_eq!(format.channels, 2);

	let output_buffer = Cursor::new(Vec::new());
	let mut writer = WavWriter::new(output_buffer, format).unwrap();

	let mut decoder = PcmDecoder::new(format);
	let timebase = Timebase::new(1, format.sample_rate);
	let mut encoder = PcmEncoder::new(timebase);

	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				if let Some(frame) = decoder.decode(packet).unwrap() {
					if let Some(pkt) = encoder.encode(frame).unwrap() {
						writer.write_packet(pkt).unwrap();
					}
				}
			}
			None => break,
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_slice_as_media_read() {
	let data: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
	let mut slice = data;

	let mut buf = [0u8; 4];
	slice.read_exact(&mut buf).unwrap();
	assert_eq!(buf, [1, 2, 3, 4]);

	slice.read_exact(&mut buf).unwrap();
	assert_eq!(buf, [5, 6, 7, 8]);
}

#[test]
fn test_vec_as_media_write() {
	let mut vec = Vec::new();

	vec.write_all(&[1, 2, 3, 4]).unwrap();
	vec.write_all(&[5, 6, 7, 8]).unwrap();

	assert_eq!(vec, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_read_primitives() {
	let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0xFE, 0xFF];
	let mut cursor = Cursor::new(data);

	let val_u16_le = cursor.read_u16_le().unwrap();
	assert_eq!(val_u16_le, 0x0201);

	let val_u32_le = cursor.read_u32_le().unwrap();
	assert_eq!(val_u32_le, 0x06050403);

	let val_i16_le = cursor.read_i16_le().unwrap();
	assert_eq!(val_i16_le, -2);
}

#[test]
fn test_write_primitives() {
	let mut cursor = Cursor::new(Vec::new());

	cursor.write_u16_le(0x0201).unwrap();
	cursor.write_u32_le(0x06050403).unwrap();
	cursor.write_i16_le(-2).unwrap();

	let data = cursor.into_inner();
	assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0xFE, 0xFF]);
}

#[test]
fn test_y4m_aspect_ratio_io() {
	let y4m_data = common::create_test_y4m_no_colorspace();
	let cursor = Cursor::new(y4m_data);

	let mut reader = Y4mReader::new(cursor).unwrap();
	let format = reader.format();

	assert!(format.aspect_ratio.is_some());
	let aspect = format.aspect_ratio.unwrap();
	assert_eq!(aspect.num, 128);
	assert_eq!(aspect.den, 117);

	let output_buffer = Cursor::new(Vec::new());
	let buf_writer: BufferedWriter<Cursor<Vec<u8>>> = BufferedWriter::new(output_buffer);
	let mut writer = Y4mWriter::new(buf_writer, format.clone()).unwrap();

	loop {
		match reader.read_packet().unwrap() {
			Some(packet) => {
				writer.write_packet(packet).unwrap();
			}
			None => break,
		}
	}

	writer.finalize().unwrap();
}

#[test]
fn test_wav_format_properties() {
	let format = WavFormat { channels: 2, sample_rate: 48000, bit_depth: 16 };

	assert_eq!(format.bytes_per_sample(), 2);
	assert_eq!(format.bytes_per_frame(), 4);
}
