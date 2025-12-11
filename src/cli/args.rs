use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ffmpreg")]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
pub struct Args {
	#[arg(short, long, value_name = "FILE", help = "Input file or glob pattern")]
	pub input: String,

	#[arg(short, long, value_name = "FILE", help = "Output file or directory")]
	pub output: Option<String>,

	#[arg(long, help = "Show frame information (like ffprobe)")]
	pub show: bool,

	#[arg(
		long = "apply",
		value_name = "FILTER",
		help = "Apply transform (e.g., gain=2.0, normalize)"
	)]
	pub transforms: Vec<String>,

	#[arg(long, value_name = "CODEC", help = "Output codec (pcm, adpcm)")]
	pub codec: Option<String>,
}

impl Args {
	pub fn parse() -> Self {
		<Self as clap::Parser>::parse()
	}
}
