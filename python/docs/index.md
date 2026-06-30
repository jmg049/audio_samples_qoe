# audio-samples-qoe

**ViSQOL** (Virtual Speech Quality Objective Listener) — a pure-Rust
full-reference perceptual audio quality metric with Python bindings.

Given a clean **reference** signal and a **degraded** signal (codec artefacts,
bandwidth limiting, noise, etc.) it returns a **MOS-LQO** (mean opinion score —
listening quality objective) in the range **1.0 – 5.0**. Higher is better.

The implementation is conformance-tested against Google's C++ reference in both
audio and speech mode. The scoring core is Rust, so the GIL is released during
computation — scoring parallelises freely across Python threads.

---

## Quick start

```python
from audio_samples_qoe import visqol

# Score two files — no Python audio I/O library needed.
score = visqol("reference.wav", "degraded.wav")
print(f"MOS-LQO: {score:.3f}")
```

```python
from audio_samples import sine_wave, white_noise
from audio_samples_qoe import visqol

ref = sine_wave(440, 5.0, 48_000)
deg = ref + white_noise(5.0, 48_000, amplitude=0.02, seed=0)

score = visqol(ref, deg)
print(f"MOS-LQO: {score:.3f}")
```

---

## Contents

```{toctree}
:maxdepth: 2

quickstart
guide/inputs
guide/modes
guide/errors
api
```
