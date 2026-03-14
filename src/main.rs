use osaka::{cli, message::Report};

fn main() {
	let args = cli::Cli::parse().report();
	cli::runner(args).report();
}
