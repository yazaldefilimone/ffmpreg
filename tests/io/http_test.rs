use ffmpreg::io::{HttpIo, Io};

const BASE_URL: &str = "https://samplefile.com/samples/download/audio/wav";
const FIXTURE_DIR: &str = "tests/fixtures/wav";

#[test]
#[ignore = "requires external network"]
fn http_io_reads_remote_wav_fixture_and_can_seek_to_start() {
	let case = fixture_case("wav_mono_8khz_tone_sample.wav");
	let url = fixture_url(case.file_name);

	let mut http = HttpIo::open(&url).unwrap();
	assert_eq!(http.size().unwrap(), case.expected_size);

	let mut first = [0u8; 12];
	http.read(&mut first).unwrap();
	assert_eq!(&first[0..4], b"RIFF");
	assert_eq!(&first[8..12], b"WAVE");

	http.seek(0).unwrap();

	let mut header = [0u8; 12];
	http.read(&mut header).unwrap();
	assert_eq!(&header[0..4], b"RIFF");
	assert_eq!(&header[8..12], b"WAVE");
}

#[test]
#[ignore = "requires external network"]
fn http_io_reads_remote_wav_fixture_payload_progressively() {
	let case = fixture_case("wav_stereo_44k_mix_sample.wav");
	let url = fixture_url(case.file_name);

	let mut http = HttpIo::open(&url).unwrap();

	let mut first = vec![0u8; 64];
	let mut second = vec![0u8; 64];

	let n1 = http.read(&mut first).unwrap();
	let n2 = http.read(&mut second).unwrap();

	assert_eq!(n1, 64);
	assert_eq!(n2, 64);
	assert_eq!(&first[0..4], b"RIFF");
	assert_ne!(first, second);
}

#[test]
#[ignore = "requires external network"]
fn http_io_rejects_missing_remote_file() {
	let url = fixture_url("wav_file_that_should_not_exist.wav");
	assert!(HttpIo::open(&url).is_err());
}

fn fixture_url(file_name: &str) -> String {
	format!("{BASE_URL}/{file_name}")
}

fn fixture_case(file_name: &'static str) -> FixtureCase {
	let path = format!("{FIXTURE_DIR}/{file_name}");
	let expected_size = std::fs::metadata(path).unwrap().len();
	FixtureCase { file_name, expected_size }
}

struct FixtureCase {
	file_name: &'static str,
	expected_size: u64,
}
