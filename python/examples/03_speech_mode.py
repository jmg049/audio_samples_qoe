"""Speech mode scoring.

Demonstrates:
- Using mode="speech" with 16 kHz signals
- The perfect-identity score of exactly 5.0
- Different noise levels and their MOS impact
- Providing 48 kHz audio to speech mode (resampled down internally)
"""

import numpy as np
from audio_samples import AudioSamples
from audio_samples_qoe import visqol

SPEECH_RATE = 16_000
DURATION = 3.0  # seconds — long enough for several speech patches

t = np.arange(int(SPEECH_RATE * DURATION)) / SPEECH_RATE
rng = np.random.default_rng(0)

# Simple voiced-speech-like signal: fundamental + two harmonics.
signal = (
    0.6 * np.sin(2 * np.pi * 200 * t)
    + 0.3 * np.sin(2 * np.pi * 400 * t)
    + 0.1 * np.sin(2 * np.pi * 600 * t)
)

def wrap(arr: np.ndarray, rate: int = SPEECH_RATE) -> AudioSamples:
    return AudioSamples.new_mono(arr.astype(np.float64), rate)

# ── identity ──────────────────────────────────────────────────────────────────

ref = wrap(signal)
score_id = visqol(ref, ref, mode="speech")
print(f"Identical (16 kHz):  MOS-LQO = {score_id:.4f}  (should be 5.0)")

# ── noise at different SNRs ───────────────────────────────────────────────────

for sigma in [0.01, 0.05, 0.10, 0.20]:
    noisy = signal + rng.normal(0, sigma, signal.shape)
    score = visqol(ref, wrap(noisy), mode="speech")
    snr = 20 * np.log10(np.std(signal) / sigma)
    print(f"Noise σ={sigma:.2f} (SNR ≈ {snr:+.0f} dB):  MOS-LQO = {score:.4f}")

# ── 48 kHz input resampled down by pipeline ───────────────────────────────────

t48 = np.arange(int(48_000 * DURATION)) / 48_000
signal_48k = (
    0.6 * np.sin(2 * np.pi * 200 * t48)
    + 0.3 * np.sin(2 * np.pi * 400 * t48)
    + 0.1 * np.sin(2 * np.pi * 600 * t48)
)
ref_48k = wrap(signal_48k, 48_000)
score_48k = visqol(ref_48k, ref_48k, mode="speech")
print(f"Identical (48 kHz input, speech mode):  MOS-LQO = {score_48k:.4f}  (should be 5.0)")
