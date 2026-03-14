use crate::cli::args::*;
use crate::{converter, message::*, play, probe};

pub fn runner(cli: Cli) -> Result<()> {
	match cli.command {
		Commands::Run(options) => converter::runner(&options),
		Commands::Play(command) => play::runner(&command),
		Commands::Probe(command) => probe::runner(&command),
	}
}
