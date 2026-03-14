use crate::probe::Builder;
use crate::probe::render::text::TextRender;
use crate::{cli, io::Input, io::Output, message};

pub fn runner(cmd: &cli::ProbeArgs) -> message::Result<()> {
	let input = Input::open(&cmd.input)?;
	let output = cmd.output.as_ref().map(|output| Output::new(output)).transpose()?;

	let builder = Builder::new(input, output);

	let media = builder.media_file()?;
	let mut renderer = TextRender::default();
	let text = renderer.render(&media);
	println!("{}", text);

	// let output = match cmd {
	// 	Some(cli::JsonOption::Pretty) => render::json::render_pretty(&media)?,
	// 	Some(cli::JsonOption::Raw) => render::json::render_raw(&media)?,
	// 	None => {

	// }
	// };
	Ok(())
}
