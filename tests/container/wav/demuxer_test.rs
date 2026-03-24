use ffmpreg::container::wav::demuxer::WavDemuxer;
use ffmpreg::core::{Demuxer, StreamId};
use ffmpreg::io::Io;

#[test]
fn wav_demuxer_reads_fmt_and_data_even_with_list_chunk() {
	let wav = wav_with_list_chunk();
	let mut demuxer = WavDemuxer::new(Box::new(CursorIo::new(wav))).unwrap();

	let stream = demuxer.streams().get(StreamId(0)).unwrap();
	assert_eq!(stream.parameters.codec.id, "pcm_s16le");
	assert_eq!(stream.parameters.sample_rate, Some(44_100));
	assert_eq!(stream.parameters.channels, Some(2));

	let packet = demuxer.read().unwrap().unwrap();
	assert_eq!(packet.data, vec![1, 2, 3, 4, 5, 6, 7, 8]);

	assert!(demuxer.read().unwrap().is_none());
}

#[test]
fn wav_demuxer_duration_and_seek_follow_audio_layout() {
	let wav = simple_wav_pcm_s16le(&[1, 2, 3, 4, 5, 6, 7, 8], 1, 2);
	let mut demuxer = WavDemuxer::new(Box::new(CursorIo::new(wav))).unwrap();

	let duration = demuxer.duration();
	assert_eq!(duration.as_seconds(), 2.0);

	demuxer.seek(1.0).unwrap();
	let packet = demuxer.read().unwrap().unwrap();
	assert_eq!(packet.data, vec![5, 6, 7, 8]);
}

#[test]
fn wav_demuxer_rejects_invalid_header() {
	let err = match WavDemuxer::new(Box::new(CursorIo::new(b"NOPE0000WAVE".to_vec()))) {
		Ok(_) => panic!("expected invalid wav header"),
		Err(err) => err,
	};
	assert!(format!("{:?}", err).contains("invalid wav header"));
}

fn simple_wav_pcm_s16le(data: &[u8], channels: u16, sample_rate: u32) -> Vec<u8> {
	let bits_per_sample = 16u16;
	let byte_rate = sample_rate * channels as u32 * 2;
	let block_align = channels * 2;
	let riff_size = 36 + data.len() as u32;

	let mut wav = Vec::new();
	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&riff_size.to_le_bytes());
	wav.extend_from_slice(b"WAVE");
	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&(16u32).to_le_bytes());
	wav.extend_from_slice(&(1u16).to_le_bytes());
	wav.extend_from_slice(&channels.to_le_bytes());
	wav.extend_from_slice(&sample_rate.to_le_bytes());
	wav.extend_from_slice(&byte_rate.to_le_bytes());
	wav.extend_from_slice(&block_align.to_le_bytes());
	wav.extend_from_slice(&bits_per_sample.to_le_bytes());
	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
	wav.extend_from_slice(data);
	wav
}

fn wav_with_list_chunk() -> Vec<u8> {
	let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
	let mut wav = Vec::new();
	let list = b"INFO";
	let riff_size = 4 + (8 + 16) + (8 + list.len() as u32) + (8 + data.len() as u32);

	wav.extend_from_slice(b"RIFF");
	wav.extend_from_slice(&riff_size.to_le_bytes());
	wav.extend_from_slice(b"WAVE");
	wav.extend_from_slice(b"fmt ");
	wav.extend_from_slice(&(16u32).to_le_bytes());
	wav.extend_from_slice(&(1u16).to_le_bytes());
	wav.extend_from_slice(&(2u16).to_le_bytes());
	wav.extend_from_slice(&(44_100u32).to_le_bytes());
	wav.extend_from_slice(&(176_400u32).to_le_bytes());
	wav.extend_from_slice(&(4u16).to_le_bytes());
	wav.extend_from_slice(&(16u16).to_le_bytes());
	wav.extend_from_slice(b"LIST");
	wav.extend_from_slice(&(list.len() as u32).to_le_bytes());
	wav.extend_from_slice(list);
	wav.extend_from_slice(b"data");
	wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
	wav.extend_from_slice(&data);
	wav
}

struct CursorIo {
	data: Vec<u8>,
	cursor: usize,
}

impl CursorIo {
	fn new(data: Vec<u8>) -> Self {
		Self { data, cursor: 0 }
	}
}

impl Io for CursorIo {
	fn read(&mut self, buf: &mut [u8]) -> ffmpreg::Result<usize> {
		if self.cursor >= self.data.len() {
			return Ok(0);
		}

		let remaining = &self.data[self.cursor..];
		let n = remaining.len().min(buf.len());
		buf[..n].copy_from_slice(&remaining[..n]);
		self.cursor += n;
		Ok(n)
	}

	fn seek(&mut self, pos: u64) -> ffmpreg::Result<()> {
		self.cursor = pos as usize;
		Ok(())
	}

	fn write(&mut self, _buf: &[u8]) -> ffmpreg::Result<usize> {
		unreachable!("write is not used in demuxer tests")
	}
}
