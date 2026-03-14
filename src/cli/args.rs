use std::path::PathBuf;

#[derive(Debug)]
pub struct Cli {
	pub command: Commands,
}

#[derive(Debug)]
pub enum Commands {
	Play(PlayArgs),
	Probe(ProbeArgs),
	Run(RunArgs),
}

#[derive(Debug)]
pub struct PlayArgs {
	pub input: PathBuf,

	pub audio: Vec<String>,
	pub video: Vec<String>,
	pub subtitle: Vec<String>,

	pub apply: Vec<String>,
}

#[derive(Debug)]
pub struct ProbeArgs {
	pub input: PathBuf,
	pub output: Option<PathBuf>,

	pub audio: Vec<String>,
	pub video: Vec<String>,
	pub subtitle: Vec<String>,

	pub apply: Vec<String>,
}

#[derive(Debug)]
pub struct RunArgs {
	pub input: PathBuf,
	pub output: PathBuf,

	pub audio: Vec<String>,
	pub video: Vec<String>,
	pub subtitle: Vec<String>,

	pub apply: Vec<String>,
}
