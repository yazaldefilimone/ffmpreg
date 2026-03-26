use ffmpreg::core::{Demuxer, Muxer};
use ffmpreg::io::{Input, Output};

#[test]
fn wav_fixtures_expose_expected_stream_metadata() {
	for case in fixture_cases() {
		let mut input = Input::open(case.path).unwrap();
		let stream = input.streams().get(0usize.into()).unwrap();

		assert_eq!(stream.parameters.codec.id, "pcm_s16le", "{}", case.path);
		assert_eq!(stream.parameters.sample_rate, Some(case.sample_rate), "{}", case.path);
		assert_eq!(stream.parameters.channels, Some(case.channels), "{}", case.path);

		let payload = read_all_packets(&mut input);
		assert!(!payload.is_empty(), "{}", case.path);

		let duration = input.duration().as_seconds();
		let expected_duration =
			payload.len() as f64 / (case.sample_rate as f64 * case.channels as f64 * 2.0);
		assert_eq!(duration, expected_duration, "{}", case.path);
	}
}

#[test]
fn wav_fixtures_seek_matches_payload_suffix() {
	for case in fixture_cases() {
		let mut input = Input::open(case.path).unwrap();
		let payload = read_all_packets(&mut input);

		let byte_rate = case.sample_rate as usize * case.channels as usize * 2;
		let block_align = case.channels as usize * 2;
		let seek_time = 0.25_f64.min(input.duration().as_seconds() / 2.0);
		let raw_offset = (seek_time * byte_rate as f64).floor() as usize;
		let aligned_offset = (raw_offset / block_align) * block_align;

		let mut input = Input::open(case.path).unwrap();
		input.seek(seek_time).unwrap();
		let suffix = read_all_packets(&mut input);

		assert_eq!(suffix, payload[aligned_offset..], "{}", case.path);
	}
}

#[test]
fn wav_fixtures_roundtrip_payload_without_loss() {
	let dir = tempfile::tempdir().unwrap();

	for case in fixture_cases() {
		let mut input = Input::open(case.path).unwrap();
		let source_stream = input.streams().get(0usize.into()).unwrap().clone();
		let source_payload = read_all_packets(&mut input);

		let output_path = dir.path().join(case.output_name);
		let mut output = Output::create(&output_path).unwrap();
		output.add(&source_stream).unwrap();
		output.write(ffmpreg::core::Packet::new(0usize.into(), source_payload.clone())).unwrap();
		output.finish().unwrap();

		let mut roundtrip = Input::open(output_path.to_str().unwrap()).unwrap();
		let roundtrip_stream = roundtrip.streams().get(0usize.into()).unwrap().clone();
		let roundtrip_payload = read_all_packets(&mut roundtrip);

		assert_eq!(
			roundtrip_stream.parameters.codec.id, source_stream.parameters.codec.id,
			"{}",
			case.path
		);
		assert_eq!(
			roundtrip_stream.parameters.sample_rate, source_stream.parameters.sample_rate,
			"{}",
			case.path
		);
		assert_eq!(
			roundtrip_stream.parameters.channels, source_stream.parameters.channels,
			"{}",
			case.path
		);
		assert_eq!(roundtrip_payload, source_payload, "{}", case.path);
	}
}

fn read_all_packets(input: &mut Input) -> Vec<u8> {
	let mut payload = Vec::new();
	while let Some(packet) = input.read().unwrap() {
		payload.extend_from_slice(&packet.data);
	}
	payload
}

fn fixture_cases() -> &'static [FixtureCase] {
	&[
		FixtureCase {
			path: "tests/fixtures/wav/wav_mono_8khz_tone_sample.wav",
			output_name: "wav_mono_8khz_tone_sample_out.wav",
			sample_rate: 8_000,
			channels: 1,
		},
		FixtureCase {
			path: "tests/fixtures/wav/wav_silence_then_tone_sample.wav",
			output_name: "wav_silence_then_tone_sample_out.wav",
			sample_rate: 16_000,
			channels: 1,
		},
		FixtureCase {
			path: "tests/fixtures/wav/wav_stereo_44k_mix_sample.wav",
			output_name: "wav_stereo_44k_mix_sample_out.wav",
			sample_rate: 44_100,
			channels: 2,
		},
		FixtureCase {
			path: "tests/fixtures/wav/wav_voice_band_16khz_sample.wav",
			output_name: "wav_voice_band_16khz_sample_out.wav",
			sample_rate: 16_000,
			channels: 1,
		},
	]
}

struct FixtureCase {
	path: &'static str,
	output_name: &'static str,
	sample_rate: u32,
	channels: u8,
}
