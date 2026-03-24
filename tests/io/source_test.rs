use ffmpreg::io::source::{Source, parse_source};

#[test]
fn parse_source_detects_file_path() {
	match parse_source("./playground/input.wav").unwrap() {
		Source::File(path) => assert_eq!(path, "./playground/input.wav"),
		Source::Url(_) => panic!("expected file source"),
	}
}

#[test]
fn parse_source_detects_http_url() {
	match parse_source("https://example.com/audio.wav").unwrap() {
		Source::Url(url) => assert_eq!(url, "https://example.com/audio.wav"),
		Source::File(_) => panic!("expected url source"),
	}
}

#[test]
fn parse_source_rejects_unknown_scheme() {
	assert!(parse_source("ftp://example.com/audio.wav").is_err());
}
