"""Sweep noise level and print a MOS vs SNR table.

Useful for sanity-checking that the metric responds monotonically to
degradation severity — a basic validation step before integrating into
a pipeline.

Usage:
    python examples/05_noise_sweep.py
    python examples/05_noise_sweep.py --mode speech --rate 16000
"""

import argparse

import numpy as np
from audio_samples import AudioSamples
from audio_samples_qoe import visqol

SIGMAS = [0.0, 0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5]


def make_signal(rate: int, duration: float = 5.0) -> np.ndarray:
    t = np.arange(int(rate * duration)) / rate
    return (
        0.6 * np.sin(2 * np.pi * 440 * t)
        + 0.3 * np.sin(2 * np.pi * 880 * t)
        + 0.1 * np.sin(2 * np.pi * 1320 * t)
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["audio", "speech"], default="audio")
    parser.add_argument("--rate", type=int, default=48_000)
    args = parser.parse_args()

    rng = np.random.default_rng(0)
    signal = make_signal(args.rate)
    ref = AudioSamples.new_mono(signal, args.rate)

    print(f"{'Sigma':>8}  {'SNR (dB)':>10}  {'MOS-LQO':>10}  mode={args.mode}")
    print("-" * 40)

    for sigma in SIGMAS:
        if sigma == 0.0:
            deg = ref
            snr_str = "    ∞"
        else:
            noisy = signal + rng.normal(0, sigma, signal.shape)
            deg = AudioSamples.new_mono(noisy, args.rate)
            snr = 20 * np.log10(np.std(signal) / sigma)
            snr_str = f"{snr:+6.1f}"

        score = visqol(ref, deg, mode=args.mode)
        sigma_str = f"{sigma:.3f}" if sigma > 0 else "0.000 (identity)"
        print(f"{sigma_str:>8}  {snr_str:>10}  {score:>10.4f}")


if __name__ == "__main__":
    main()
