<div align="center">

# AudioSamples QoE

## Quality of Experience Metrics on AudioSamples


<img src="https://raw.githubusercontent.com/jmg049/audio_samples_qoe/main/logo.png" title="AudioSamples Logo -- Ferrous' Mustachioed Cousin From East Berlin, Eisenhaltig" width="200"/>

[![PyPI][pypi-img]][pypi] [![Python Docs][pydocs-img]][pydocs] [![License: MIT][license-img]][license]
</div>

Perceptual audio quality metrics in pure Rust, with Python bindings.

**ViSQOL** (Virtual Speech Quality Objective Listener) is a full-reference
metric that compares a degraded signal against a clean reference and predicts
a MOS-LQO in the range 1.0 to 5.0. Audio and speech modes, conformance-tested
against the reference implementation.

Built on the [`audio_samples`](https://pypi.org/project/audio-samples/)
ecosystem.

> The crate also ships SCOREQ, a neural speech-quality metric. It is not yet
> exposed in the Python bindings.

## Install

```sh
pip install audio-samples-qoe
```

The wheel is self-contained, so no Rust toolchain is needed. To build from
source instead, install [maturin](https://maturin.rs) and run `maturin develop
--release` from the repository root.

## Usage

```python
from audio_samples import AudioSamples
from audio_samples_qoe import visqol

# Files or audio_samples.AudioSamples, in any combination.
score = visqol("reference.wav", "degraded.wav")

# Speech mode is a keyword away.
score = visqol("reference.wav", "degraded.wav", mode="speech")
```

Each signal may be an `AudioSamples` object or a path to a WAV/FLAC file
(decoded by the native Rust reader), in any combination. Raw arrays are
deliberately not accepted: wrap them with `AudioSamples.new_mono(arr,
sample_rate)` / `new_multi`, which is where the sample rate is attached.
Multi-channel signals are mixed down to mono, and any input sample rate is
handled internally. `visqol` releases the GIL while computing, so scoring
parallelises across Python threads.

## Modes

| Mode       | Operating rate            | Bands            | MOS mapping              | Patch selection      |
|------------|---------------------------|------------------|--------------------------|----------------------|
| `"audio"`  | 48 kHz (resampled)        | 32, up to Nyquist| Support vector regression| All patches          |
| `"speech"` | Reference's native rate   | 21, ≤ 8 kHz      | Exponential NSIM fit     | Voice-activity gated |

In speech mode the reference is never resampled (16 kHz input is recommended),
silence in the reference is excluded by a voice-activity detector, and
identical signals score 5.0.

## Errors

- `TypeError`: an argument is neither an `AudioSamples` nor a path.
- `ValueError`: invalid `mode` string.
- `OSError`: a file could not be read or decoded.
- `audio_samples_qoe.VisqolError`: the metric pipeline failed (most
  commonly a signal too short to extract a single analysis patch).

The package ships full type stubs and a `py.typed` marker, so `mypy`,
`pyright`, and IDE completion work out of the box.

## Changelog

See [CHANGELOG.md](https://github.com/jmg049/audio_samples_qoe/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/jmg049/audio_samples_qoe/blob/main/CONTRIBUTING.md).

## License

MIT

[pypi]: https://pypi.org/project/audio-samples-qoe/
[pypi-img]: https://img.shields.io/pypi/v/audio-samples-qoe?style=for-the-badge&color=009E73&label=pypi
[pydocs]: https://jmg049.github.io/audio_samples_qoe/
[pydocs-img]: https://img.shields.io/badge/python%20docs-online-009E73?style=for-the-badge&labelColor=gray
[license-img]: https://img.shields.io/crates/l/audio_samples_qoe?style=for-the-badge&label=license&labelColor=gray
[license]: https://github.com/jmg049/audio_samples_qoe/blob/main/LICENSE
