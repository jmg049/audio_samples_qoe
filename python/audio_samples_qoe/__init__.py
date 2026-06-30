"""Perceptual audio quality metrics, implemented in Rust.

This package provides ViSQOL (Virtual Speech Quality Objective Listener), a
full-reference perceptual quality metric that compares a degraded signal
against a clean reference and predicts a MOS-LQO (mean opinion score —
listening quality objective) in the range 1.0–5.0.

The single entry point is :func:`visqol`. It accepts, for each signal,
either an :class:`audio_samples.AudioSamples` object or a path to an audio
file (WAV or FLAC, decoded with the native Rust reader) — in any
combination. Raw sample arrays should be wrapped first with
``audio_samples.AudioSamples.new_mono`` / ``new_multi``, which is also where
the sample rate is attached.

Example:
    >>> from audio_samples import sine_wave, white_noise
    >>> from audio_samples_qoe import visqol
    >>> ref = sine_wave(440, 5.0, 48000)
    >>> deg = ref + white_noise(5.0, 48000, amplitude=0.01, seed=0)
    >>> visqol(ref, deg)  # doctest: +SKIP
    4.4...
    >>> visqol("reference.wav", "degraded.wav")  # doctest: +SKIP
    3.2...
"""

from os import PathLike, fspath
from typing import Literal, TypeAlias, Union

import numpy as np
import numpy.typing as npt
from audio_samples import AudioSamples

from audio_samples_qoe._native import VisqolError, __version__, _read, _score

VisqolMode: TypeAlias = Literal["audio", "speech"]
"""Scoring mode accepted by :func:`visqol`: ``"audio"`` (full-band, 32
gammatone bands up to Nyquist) or ``"speech"`` (21 bands capped at 8 kHz,
voice-activity-gated patch selection)."""

AudioInput: TypeAlias = Union[AudioSamples, str, "PathLike[str]"]
"""A signal argument to :func:`visqol`: an in-memory
:class:`audio_samples.AudioSamples` or a path to a WAV/FLAC file."""

__all__ = [
    "AudioInput",
    "VisqolError",
    "VisqolMode",
    "__version__",
    "visqol",
]


def _coerce(signal: AudioInput, name: str) -> "tuple[npt.NDArray[np.float64], int]":
    """Normalise a :func:`visqol` argument to ``(float64 samples, rate)``."""
    if isinstance(signal, AudioSamples):
        # as_f64() normalises integer sample formats to [-1, 1].
        f64 = signal.as_f64()
        return np.asarray(f64, dtype=np.float64), f64.sample_rate
    if isinstance(signal, (str, PathLike)):
        return _read(fspath(signal))
    raise TypeError(
        f"{name} must be an audio_samples.AudioSamples, str, or os.PathLike, "
        f"got {type(signal).__name__}; wrap raw sample arrays with "
        f"audio_samples.AudioSamples.new_mono(arr, sample_rate) or new_multi"
    )


def visqol(
    reference: AudioInput,
    degraded: AudioInput,
    *,
    mode: VisqolMode = "audio",
) -> float:
    """Compute the ViSQOL MOS-LQO score between a reference and a degraded signal.

    ViSQOL (Virtual Speech Quality Objective Listener) is a full-reference
    perceptual quality metric: it compares a degraded signal against a clean
    reference and predicts a mean opinion score.

    Each signal may be given as an :class:`audio_samples.AudioSamples`
    object or as a path to a WAV or FLAC file (decoded with the native Rust
    reader — no Python audio I/O library is needed), in any combination.
    Multi-channel signals are mixed down to mono by channel averaging, and
    any sample type is accepted.

    In ``"audio"`` mode both signals are resampled internally to the 48 kHz
    rate the model is calibrated for, and the score is predicted by a
    support vector regression over 32 gammatone bands. In ``"speech"`` mode
    the pipeline matches the C++ reference: it runs at the reference
    signal's native sample rate (the degraded signal is resampled to match)
    with 21 gammatone bands capped at 8 kHz, reference patches are gated on
    voice activity so silence is excluded, and the score comes from an
    exponential NSIM→MOS fit (identical signals map to 5.0). Upstream
    recommends 16 kHz input for speech mode.

    The GIL is released while the score is computed, so scoring parallelises
    across Python threads.

    Args:
        reference: Clean reference signal — an ``AudioSamples`` or a
            WAV/FLAC file path.
        degraded: Degraded signal to evaluate — an ``AudioSamples`` or a
            WAV/FLAC file path.
        mode: ``"audio"`` (default) for full-band audio, ``"speech"`` for
            speech.

    Returns:
        The MOS-LQO score, a float in [1.0, 5.0]. Higher is better.
        Identical signals score ~4.73 in audio mode and 5.0 in speech mode.

    Raises:
        TypeError: If an argument is neither an ``AudioSamples`` nor a path.
        ValueError: If ``mode`` is not ``"audio"`` or ``"speech"``.
        OSError: If a file path cannot be read or decoded.
        VisqolError: If the metric pipeline fails — most commonly because a
            signal is too short to extract a single analysis patch
            (roughly 0.6 s in audio mode, 0.5 s in speech mode).

    Example:
        >>> from audio_samples import sine_wave
        >>> from audio_samples_qoe import visqol
        >>> ref = sine_wave(440, 5.0, 48000)
        >>> score = visqol(ref, "degraded.wav")  # doctest: +SKIP
    """
    ref_samples, ref_rate = _coerce(reference, "reference")
    deg_samples, deg_rate = _coerce(degraded, "degraded")
    return _score(ref_samples, ref_rate, deg_samples, deg_rate, mode)
