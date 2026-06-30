<div align="center">

# AudioSamples QoE

## Quality of Experience Metrics on AudioSamples

<img src="https://raw.githubusercontent.com/jmg049/audio_samples_qoe/main/logo.png" title="AudioSamples Logo -- Ferrous' Mustachioed Cousin From East Berlin, Eisenhaltig" width="200"/>

[![Crates.io][crate-img]][crate] [![Docs.rs][docs-img]][docs]
[![PyPI][pypi-img]][pypi] [![Python Docs][pydocs-img]][pydocs]
[![MSRV][msrv-img]][msrv] [![License: MIT][license-img]][license]

</div>

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

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT

[crate]: https://crates.io/crates/audio_samples_qoe
[crate-img]: https://img.shields.io/crates/v/audio_samples_qoe?style=for-the-badge&color=009E73&label=crates.io
[docs]: https://docs.rs/audio_samples_qoe
[docs-img]: https://img.shields.io/badge/docs.rs-online-009E73?style=for-the-badge&labelColor=gray
[pypi]: https://pypi.org/project/audio-samples-qoe/
[pypi-img]: https://img.shields.io/pypi/v/audio-samples-qoe?style=for-the-badge&color=009E73&label=pypi
[pydocs]: https://jmg049.github.io/audio_samples_qoe/
[pydocs-img]: https://img.shields.io/badge/python%20docs-online-009E73?style=for-the-badge&labelColor=gray
[msrv]: https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/
[msrv-img]: https://img.shields.io/badge/MSRV-1.87%2B-009E73?style=for-the-badge&labelColor=gray
[license-img]: https://img.shields.io/crates/l/audio_samples_qoe?style=for-the-badge&label=license&labelColor=gray
[license]: https://github.com/jmg049/audio_samples_qoe/blob/main/LICENSE
