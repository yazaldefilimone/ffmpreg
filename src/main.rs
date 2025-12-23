use ffmpreg::cli::{Args, BatchPipeline, Pipeline, is_batch_pattern, is_directory};
use ffmpreg::show::{Show, ShowOptions};

fn main() {
	let args = Args::parse();

	let result = if args.show {
		let opts = ShowOptions {
			json: args.json,
			stream_filter: args.stream,
			frame_limit: args.frames,
			hex_limit: args.hex_limit,
		};
		let show = Show::new(args.input.clone(), opts);
		show.run()
	} else if is_batch_pattern(&args.input) {
		let output_dir = args.output.clone().unwrap_or_else(|| "out".to_string());
		let batch = BatchPipeline::new(args.input.clone(), output_dir, false, args.transforms.clone());
		batch.run()
	} else if args.output.as_ref().map(|o| is_directory(o)).unwrap_or(false) {
		let output_dir = args.output.clone().unwrap();
		let batch = BatchPipeline::new(args.input.clone(), output_dir, false, args.transforms.clone());
		batch.run()
	} else {
		let pipeline =
			Pipeline::new(args.input.clone(), args.output.clone(), false, args.transforms.clone());
		pipeline.run()
	};

	match result {
		Ok(()) => {
			if !args.show {
				if let Some(output) = &args.output {
					println!("ok: {} -> {}", args.input, output);
				}
			}
		}
		Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	}
}
