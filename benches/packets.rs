use ffmpeg_next as ffmpeg;
use ffmpreg::core::traits::*;
use ffmpreg::io::Input;
use std::time::Instant;

const INPUT: &str = "./playground/your_name_sparkle.wav";

fn ffmpeg_run() -> (usize, usize) {
	let mut ictx = ffmpeg::format::input(INPUT).unwrap();

	let mut packets = 0;
	let mut bytes = 0;

	for p in ictx.packets() {
		bytes += p.1.size() as usize;
		packets += 1;
	}

	(packets, bytes)
}

fn ffmpreg_run() -> (usize, usize) {
	let mut demuxer = Input::open(INPUT).unwrap();

	let mut packets = 0;
	let mut bytes = 0;

	while let Some(p) = demuxer.read().unwrap() {
		bytes += p.data.len();
		packets += 1;
	}

	(packets, bytes)
}

fn format_rate(rate: f64) -> String {
	if rate >= 1_000_000_000.0 {
		format!("{:.2} B", rate / 1_000_000_000.0)
	} else if rate >= 1_000_000.0 {
		format!("{:.2} M", rate / 1_000_000.0)
	} else if rate >= 1_000.0 {
		format!("{:.2} K", rate / 1_000.0)
	} else {
		format!("{:.2}", rate)
	}
}

fn run(name: &str, f: fn() -> (usize, usize), runs: usize) {
	let mut total_packets = 0;
	let mut total_bytes = 0;

	let start = Instant::now();
	for _ in 0..runs {
		let (p, b) = f();
		total_packets += p;
		total_bytes += b;
	}
	let elapsed = start.elapsed().as_secs_f64();

	let pps = total_packets as f64 / elapsed;
	let bps = total_bytes as f64 / elapsed;

	println!(
		"{name}: time={:.3}s packets/s={} bytes/s={}",
		elapsed,
		format_rate(pps),
		format_rate(bps)
	);
}

fn main() {
	ffmpeg::init().unwrap();

	let runs = 10;

	run("ffmpeg", ffmpeg_run, runs);
	run("ffmpreg", ffmpreg_run, runs);
}
