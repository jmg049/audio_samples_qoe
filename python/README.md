# audio-samples-qoe

Python bindings for a pure-Rust implementation of **ViSQOL** (Virtual Speech
Quality Objective Listener) — a full-reference perceptual audio quality metric
that predicts a MOS-LQO score in the range 1.0–5.0 from a clean reference and
a degraded signal.

The implementation is conformance-tested against Google's C++ reference in
both audio and speech mode. Part of the
[`audio_samples`](https://pypi.org/project/audio-samples/) ecosystem.

## Installation

```sh
pip install audio-samples-qoe
```

Or build from source (requires Rust and [maturin](https://maturin.rs)):

```sh
cd python
maturin develop --release
```

## Usage

The single entry point is `visqol(reference, degraded, *, mode="audio")`.
Each signal may be an `audio_samples.AudioSamples` object or a path to a
WAV/FLAC file (decoded by the native Rust reader), in any combination:

```python
import numpy as np
from audio_samples import AudioSamples
from audio_samples_qoe import visqol

# From files — no Python audio I/O library needed.
score = visqol("reference.wav", "degraded.wav")

# From in-memory signals.
t = np.arange(48000 * 5) / 48000
clean = np.sin(2 * np.pi * 440 * t)
noisy = clean + np.random.default_rng(0).normal(0, 0.01, clean.shape)
ref = AudioSamples.new_mono(clean, 48000)
deg = AudioSamples.new_mono(noisy, 48000)
score = visqol(ref, deg)

# Mixed is fine too, and speech mode is a keyword away.
score = visqol("reference.wav", deg, mode="speech")
```

Raw arrays are deliberately not accepted — wrap them with
`AudioSamples.new_mono(arr, sample_rate)` / `new_multi`, which is where the
sample rate is attached. Multi-channel signals are mixed down to mono, and
any input sample rate is handled internally.

`visqol` releases the GIL while computing, so scoring parallelises across
Python threads.

## Modes

| Mode       | Operating rate            | Bands            | MOS mapping              | Patch selection |
|------------|---------------------------|------------------|--------------------------|-----------------|
| `"audio"`  | 48 kHz (resampled)        | 32, up to Nyquist| Support vector regression| All patches     |
| `"speech"` | Reference's native rate   | 21, ≤ 8 kHz      | Exponential NSIM fit     | Voice-activity gated |

Speech mode matches the C++ reference: it never resamples the reference
(upstream recommends 16 kHz input), silence in the reference is excluded
via an RMS voice-activity detector, and identical signals score 5.0.

## Errors

- `TypeError` — an argument is neither an `AudioSamples` nor a path.
- `ValueError` — invalid `mode` string.
- `OSError` — a file could not be read or decoded.
- `audio_samples_qoe.VisqolError` — the metric pipeline failed (most
  commonly: a signal too short to extract a single analysis patch).

The package ships full type stubs and a `py.typed` marker, so `mypy`,
`pyright`, and IDE completion work out of the box.
