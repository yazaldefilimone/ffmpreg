use ffmpreg::io::{File, Io};

#[test]
fn file_io_can_write_seek_and_read_back() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("sample.bin");

	let mut file = File::create(&path).unwrap();
	file.write_all(b"abcdef").unwrap();
	file.seek(2).unwrap();
	file.write_all(b"XY").unwrap();

	let size = file.size().unwrap();
	assert_eq!(size, 6);

	let mut file = File::open(&path).unwrap();
	let mut data = [0u8; 6];
	file.read(&mut data).unwrap();
	assert_eq!(&data, b"abXYef");
}
