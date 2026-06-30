"""Score signals constructed from NumPy arrays.

Demonstrates:
- Creating AudioSamples from float64, float32, and int16 arrays
- Stereo input (mixed to mono internally)
- Noise degradation
"""

import numpy as np
from audio_samples import AudioSamples
from audio_samples_qoe import visqol

RATE = 48_000
DURATION = 5.0
t = np.arange(int(RATE * DURATION)) / RATE
rng = np.random.default_rng(42)

# ── float64 mono ─────────────────────────────────────────────────────────────

clean_f64 = np.sin(2 * np.pi * 440 * t)
noisy_f64 = clean_f64 + rng.normal(0, 0.05, clean_f64.shape)

ref = AudioSamples.new_mono(clean_f64, RATE)
deg = AudioSamples.new_mono(noisy_f64, RATE)

score = visqol(ref, deg)
print(f"float64 mono,  σ=0.05:  MOS-LQO = {score:.4f}")

# ── float32 mono ─────────────────────────────────────────────────────────────

clean_f32 = clean_f64.astype(np.float32)
noisy_f32 = (clean_f64 + rng.normal(0, 0.05, clean_f64.shape)).astype(np.float32)

score = visqol(
    AudioSamples.new_mono(clean_f32, RATE),
    AudioSamples.new_mono(noisy_f32, RATE),
)
print(f"float32 mono,  σ=0.05:  MOS-LQO = {score:.4f}")

# ── int16 PCM ─────────────────────────────────────────────────────────────────

clean_i16 = (clean_f64 * 32767).astype(np.int16)
noisy_i16 = np.clip(clean_i16 + rng.integers(-1638, 1638, clean_i16.shape, dtype=np.int16), -32768, 32767).astype(np.int16)

score = visqol(
    AudioSamples.new_mono(clean_i16, RATE),
    AudioSamples.new_mono(noisy_i16, RATE),
)
print(f"int16 PCM,     ~5% noise: MOS-LQO = {score:.4f}")

# ── stereo input (mixed to mono) ─────────────────────────────────────────────

left  = np.sin(2 * np.pi * 440 * t)
right = np.sin(2 * np.pi * 880 * t)
stereo_clean = np.stack([left, right])                            # shape (2, N)
stereo_noisy = stereo_clean + rng.normal(0, 0.05, stereo_clean.shape)

score = visqol(
    AudioSamples.new_multi(stereo_clean, RATE),
    AudioSamples.new_multi(stereo_noisy, RATE),
)
print(f"stereo (2ch),  σ=0.05:  MOS-LQO = {score:.4f}")

# ── identity: same signal ─────────────────────────────────────────────────────

identical_score = visqol(ref, ref)
print(f"identical:               MOS-LQO = {identical_score:.4f}  (ceiling ~4.73)")
