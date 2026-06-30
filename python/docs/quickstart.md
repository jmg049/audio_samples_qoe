# Quickstart

## Installation

```sh
pip install audio-samples-qoe
```

Or build from source (requires Rust ≥ 1.80 and [maturin](https://maturin.rs)):

```sh
git clone ...
cd python
maturin develop --release
```

---

## Scoring two files

The simplest usage: pass two file paths and read the score back.

```python
from audio_samples_qoe import visqol

score = visqol("reference.wav", "degraded.wav")
print(f"MOS-LQO: {score:.3f}")   # e.g. 3.842
```

Files are decoded with the native Rust reader — no Python audio library is
needed. WAV and FLAC are supported. Any sample rate and channel count is
accepted; the pipeline mixes down to mono and resamples internally.

---

## Scoring in-memory signals

Build signals with the {mod}`audio_samples` generators — they return
{class}`~audio_samples.AudioSamples` already carrying a sample rate, so you can
pass them straight in:

```python
from audio_samples import sine_wave, white_noise
from audio_samples_qoe import visqol

RATE = 48_000

ref = sine_wave(440, 5.0, RATE)
deg = ref + white_noise(5.0, RATE, amplitude=0.02, seed=0)

score = visqol(ref, deg)
```

Raw arrays without a sample rate are deliberately rejected — the error message
tells you exactly what to do:

```python
visqol(ref.to_numpy(), deg.to_numpy())
# TypeError: reference must be an audio_samples.AudioSamples, str, or
# os.PathLike ... wrap raw sample arrays with
# audio_samples.AudioSamples.new_mono(arr, sample_rate) or new_multi
```

---

## Mixing paths and in-memory signals

The two arguments are independent — any combination works:

```python
from pathlib import Path

score = visqol(Path("reference.wav"), deg)       # path + AudioSamples
score = visqol(ref, "degraded.wav")              # AudioSamples + path
score = visqol("reference.wav", "degraded.wav")  # both paths
score = visqol(ref, deg)                         # both in-memory
```

---

## Parallel scoring

The GIL is released while ViSQOL computes, so you can score many pairs
concurrently with `concurrent.futures.ThreadPoolExecutor`:

```python
from concurrent.futures import ThreadPoolExecutor
from audio_samples_qoe import visqol

pairs = [("ref1.wav", "deg1.wav"), ("ref2.wav", "deg2.wav"), ...]

with ThreadPoolExecutor() as pool:
    scores = list(pool.map(lambda p: visqol(*p), pairs))
```
