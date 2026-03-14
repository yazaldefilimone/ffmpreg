use crate::cli;
use crate::core::graph::Graph;
use crate::core::graph::nodes::{DecoderNode, EncoderNode, ResamplerNode};
use crate::core::resampler::AudioResampler;
use crate::core::resolver::{CodecResolver, ContainerResolver};
use crate::core::traits::Resampler;
use crate::io::{Input, Output};
use crate::message::Result;

pub fn runner(options: &cli::RunArgs) -> Result<()> {
	let containers = ContainerResolver::new();
	let codecs = CodecResolver::new();

	let mut input = Input::from_resolver(&options.input, &containers)?;
	let mut builder = Output::builder(&options.output, &containers)?;

	let mut graph = Graph::new();

	for option in options.stream_options()? {
		builder.format_codec(&option.codec)?;

		for track in input.tracks.audio_selector(&option.selector)? {
			let decoder = codecs.decoder_for(track)?;
			let encoder = codecs.encoder_for(track, builder.format_mut())?;

			let decoder_id = graph.add(DecoderNode::new(decoder));
			let mut after_decode = decoder_id;

			let format_out = encoder.format();
			let format_in = track.format;

			if let (Some(format_in), Some(format_out)) = (format_in.audio(), format_out.audio()) {
				let resampler = AudioResampler::new(*format_in, *format_out);
				if resampler.needed() {
					let resampler = ResamplerNode::new(Box::new(resampler));
					let resample_id = graph.add(resampler);
					graph.link(after_decode, resample_id);
					after_decode = resample_id;
				}
			};

			let encoder_id = graph.add(EncoderNode::new(encoder));
			graph.link(after_decode, encoder_id);

			graph.set(track.id, decoder_id);
		}
	}

	let mut output = builder.build()?;

	while let Some(packet) = input.read_packet()? {
		let packets = graph.run(packet)?;
		output.write_all(packets)?;
	}

	for packet in graph.flush()? {
		output.write_packet(packet)?;
	}

	output.finalize()
}
