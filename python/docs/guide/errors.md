# Error handling

{func}`~audio_samples_qoe.visqol` raises one of four exception types depending
on what went wrong.

---

## `TypeError` — wrong argument type

Raised when a signal argument is neither an `AudioSamples` nor a path.

```python
import numpy as np
from audio_samples_qoe import visqol

arr = np.zeros(48_000 * 5)
visqol(arr, arr)
# TypeError: reference must be an audio_samples.AudioSamples, str, or
# os.PathLike, got ndarray; wrap raw sample arrays with
# audio_samples.AudioSamples.new_mono(arr, sample_rate) or new_multi
```

**Fix**: wrap the array with `AudioSamples.new_mono(arr, sample_rate)`.

---

## `ValueError` — invalid `mode`

Raised when `mode` is not `"audio"` or `"speech"`.

```python
visqol(ref, deg, mode="music")
# ValueError: mode must be 'audio' or 'speech', got 'music'
```

---

## `OSError` — file not found or decode error

Raised when a file path cannot be opened or the format is not supported.

```python
visqol("does_not_exist.wav", "degraded.wav")
# OSError: No such file or directory (os error 2): does_not_exist.wav
```

---

## `VisqolError` — pipeline failure

Raised when the metric pipeline itself fails. The most common cause is a
signal that is too short to extract a single analysis patch.

```python
from audio_samples import AudioSamples
from audio_samples_qoe import visqol, VisqolError
import numpy as np

short = AudioSamples.new_mono(np.zeros(1000), 48_000)  # ~20 ms — far too short
try:
    visqol(short, short)
except VisqolError as e:
    print(f"Pipeline failed: {e}")
```

**Minimum signal length** (approximate):
- Audio mode: ~0.6 s at 48 kHz
- Speech mode: ~0.5 s at 16 kHz

---

## Defensive pattern

```python
from audio_samples_qoe import visqol, VisqolError

def safe_score(ref_path: str, deg_path: str) -> float | None:
    try:
        return visqol(ref_path, deg_path)
    except (OSError, VisqolError) as e:
        print(f"Skipping {deg_path}: {e}")
        return None
```
