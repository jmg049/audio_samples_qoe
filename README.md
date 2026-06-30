# audio_samples_qoe

Perceptual audio quality metrics in pure Rust, with Python bindings.

- **ViSQOL** (Virtual Speech Quality Objective Listener): a full-reference
  metric that compares a degraded signal against a clean reference and predicts
  a MOS-LQO in the range 1.0 to 5.0. Audio and speech modes, conformance-tested
  against the reference implementation.
- **SCOREQ** (Ragano et al., NeurIPS 2024): a neural speech-quality metric run
  via ONNX Runtime, with no-reference and reference modes. Optional, behind the
  `scoreq` feature.

Built on the [`audio_samples`](https://crates.io/crates/audio_samples)
ecosystem.

## Install

### Rust

```sh
cargo add audio_samples_qoe
```

SCOREQ pulls an ONNX runtime and downloads model weights on first use, so it is
opt-in:

```sh
cargo add audio_samples_qoe --features scoreq
```

### Python

```sh
pip install audio-samples-qoe
```

The wheel is self-contained, so no Rust toolchain is needed. To build from
source instead, install [maturin](https://maturin.rs) and run `maturin develop
--release`.

## Usage

### Rust

```rust
use audio_samples_io::read;
use audio_samples_qoe::{AudioSamplesQoE, VisqolOptions};

let reference = read::<_, f32>("reference.wav")?;
let degraded = read::<_, f32>("degraded.wav")?;

// Audio mode (default).
let mos = reference.visqol(&degraded)?;

// Speech mode (21 bands ≤ 8 kHz, voice-activity gated).
let mos = reference.visqol_with_options(&degraded, &VisqolOptions::speech())?;
```

SCOREQ (with `--features scoreq`):

```rust
use audio_samples_qoe::{Scoreq, ScoreqDomain, ScoreqMode};

// Construct once and reuse; loading the model is expensive.
let mut model = Scoreq::new(ScoreqDomain::Natural, ScoreqMode::Nr)?;
let mos = model.predict(&test)?;
```

### Python

```python
from audio_samples_qoe import visqol

# Files or audio_samples.AudioSamples, in any combination.
score = visqol("reference.wav", "degraded.wav")
score = visqol("reference.wav", "degraded.wav", mode="speech")
```

See [`python/README.md`](python/README.md) and the
[documentation](python/docs/) for the Python API.

## Modes

| Mode     | Operating rate          | Bands              | MOS mapping               | Patch selection      |
|----------|-------------------------|--------------------|---------------------------|----------------------|
| `audio`  | 48 kHz (resampled)      | 32, up to Nyquist  | Support vector regression | All patches          |
| `speech` | Reference's native rate | 21, ≤ 8 kHz        | Exponential NSIM fit      | Voice-activity gated |

## License

MIT
