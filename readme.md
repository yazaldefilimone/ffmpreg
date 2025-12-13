A Rust-native alternative to FFmpeg.

FFmpReg is a media processing engine written in Rust.
It provides the same core capabilities as FFmpeg like decode, transform, encode...

Below is a minimal end-to-end example in Rust. It reads a WAV file, applies gain
and normalization at the frame level, then writes a new WAV file.

```rust
use ffmpreg::prelude::*;

fn main() -> Result<()> {
    pipeline()
        .input("input.wav")
        .map(|frame| frame.gain(2.0))
        .map(|frame| frame.normalize())
        .output("output.wav")
        .run()
}
```

Frames are explicit values in the pipeline. Transforms are normal Rust functions
applied in sequence, and execution is deterministic at compile time.

The same pipeline can be expressed using the CLI with an equivalent structure:

```bash
ffmpreg -i input.wav -o output.wav \
  --apply gain=2.0 \
  --apply normalize
```

Using FFMPREG from the CLI does not introduce any additional abstraction. Each
`--apply` corresponds to a frame transform executed in order, and the output is
always a valid, immediately playable file.

Beyond simple pipelines, FFMPREG makes it easy to inspect media without switching
tools. Frame metadata can be printed directly from the decode stage, providing a
lightweight alternative to `ffprobe`:

```bash
ffmpreg -i input.wav --show
```

```
Frame 0: pts=0, samples=1024, channels=2, rate=44100
Frame 1: pts=1024, samples=1024, channels=2, rate=44100
```

Batch processing follows the same principles. Multiple inputs are treated as
independent pipelines with no shared state. This makes parallel execution and
scripting straightforward:

```bash
ffmpreg --input folder/*.wav --output out/
```

The library API exposes the same primitives at a lower level for full control.
Containers, codecs and frames are explicit types, allowing custom pipelines
without the high-level builder:

```rust
use ffmpreg::containers::wav;
use ffmpreg::codecs::pcm;

let packets = wav::read("input.wav")?;
let frames = pcm::decode_packets(&packets)?;

let frames = frames
    .into_iter()
    .map(|f| f.gain(2.0))
    .collect::<Vec<_>>();

let out = pcm::encode_frames(&frames)?;
wav::write("output.wav", &out)?;
```
