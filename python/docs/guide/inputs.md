# Input types

{func}`~audio_samples_qoe.visqol` accepts three kinds of input for each signal
argument. Any combination of the three types is valid across the two arguments.

---

## File paths

Pass a `str` or any `os.PathLike` (including `pathlib.Path`). The file is
decoded by the native Rust reader — no Python audio library is needed.

```python
from pathlib import Path
from audio_samples_qoe import visqol

score = visqol("reference.wav", Path("degraded.flac"))
```

**Supported formats**: WAV (PCM 16/24/32-bit, float) and FLAC.

---

## `audio_samples.AudioSamples`

For in-memory signals, wrap a NumPy array with
{class}`~audio_samples.AudioSamples` to attach a sample rate.

```python
import numpy as np
from audio_samples import AudioSamples

# Mono: 1-D float64 array + sample rate
arr = np.sin(2 * np.pi * 440 * np.arange(48_000 * 5) / 48_000)
signal = AudioSamples.new_mono(arr, 48_000)

# Stereo: 2-D array of shape (channels, samples) + sample rate
stereo = np.stack([arr, arr * 0.8])
signal = AudioSamples.new_multi(stereo, 48_000)
```

Any NumPy-compatible integer or float dtype is accepted. Integer formats (e.g.
`int16`, `int32`) are normalised to `[-1, 1]` internally before scoring.

```python
pcm = (arr * 32767).astype(np.int16)
signal = AudioSamples.new_mono(pcm, 48_000)
```

---

## Sample rates

Signals at any sample rate are accepted. In **audio mode** both signals are
resampled to 48 kHz internally. In **speech mode** the reference runs at its
native rate and the degraded signal is resampled to match — upstream recommends
providing 16 kHz speech input directly.

```python
ref = AudioSamples.new_mono(ref_arr, 48_000)
deg = AudioSamples.new_mono(deg_arr, 16_000)   # resampled to 48 kHz in audio mode
score = visqol(ref, deg)
```

---

## Channel count

Any channel count is accepted. All signals are mixed down to mono by channel
averaging before scoring — this matches the C++ reference behaviour.

---

## What is *not* accepted

Raw NumPy arrays without a sample rate are rejected with a `TypeError` that
explains how to fix it:

```python
import numpy as np
arr = np.zeros(48_000 * 5)
visqol(arr, arr)
# TypeError: reference must be an audio_samples.AudioSamples, str, or
# os.PathLike, got ndarray; wrap raw sample arrays with
# audio_samples.AudioSamples.new_mono(arr, sample_rate) or new_multi
```
