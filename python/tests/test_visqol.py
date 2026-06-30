"""End-to-end tests for the audio_samples_qoe Python bindings."""

from pathlib import Path

import numpy as np
import pytest
from audio_samples import AudioSamples

import audio_samples_qoe
from audio_samples_qoe import VisqolError, visqol

REPO_ROOT = Path(__file__).resolve().parents[2]
TESTDATA = REPO_ROOT / "visqol" / "testdata"
CONFORMANCE = TESTDATA / "conformance_testdata_subset"
CLEAN_SPEECH = TESTDATA / "clean_speech"

RATE = 48_000


def sine(freq: float = 440.0, seconds: float = 5.0, rate: int = RATE) -> np.ndarray:
    t = np.arange(int(rate * seconds)) / rate
    return np.sin(2 * np.pi * freq * t)


def mono(samples: np.ndarray, rate: int = RATE) -> AudioSamples:
    return AudioSamples.new_mono(samples, rate)


# ── AudioSamples inputs ──────────────────────────────────────────────────────


def test_identical_signals_score_near_perfect():
    x = mono(sine())
    score = visqol(x, x)
    assert 4.5 < score <= 5.0


def test_noisy_signal_scores_lower_than_identical():
    rng = np.random.default_rng(0)
    ref = sine()
    deg = ref + rng.normal(0.0, 0.1, ref.shape)
    noisy = visqol(mono(ref), mono(deg))
    clean = visqol(mono(ref), mono(ref))
    assert 1.0 <= noisy <= 5.0
    assert noisy < clean


def test_stereo_input_accepted():
    stereo = AudioSamples.new_multi(np.stack([sine(), sine()]), RATE)
    score = visqol(stereo, stereo)
    assert 4.5 < score <= 5.0


def test_int16_samples_accepted():
    pcm = (sine() * 32767).astype(np.int16)
    x = mono(pcm)
    assert visqol(x, x) > 4.5


def test_differing_sample_rates_resampled_internally():
    ref = mono(sine(rate=48_000), 48_000)
    deg = mono(sine(rate=24_000), 24_000)
    score = visqol(ref, deg)
    assert 1.0 <= score <= 5.0


# ── speech mode ──────────────────────────────────────────────────────────────


def test_speech_mode_identical_signals_score_perfect():
    # The exponential NSIM->MOS fit scales a perfect NSIM of 1.0 to MOS 5.0.
    x = mono(sine(220.0, rate=16_000), 16_000)
    assert visqol(x, x, mode="speech") == pytest.approx(5.0)


def test_speech_mode_degradation_lowers_score():
    rng = np.random.default_rng(1)
    ref = sine(220.0, rate=16_000)
    deg = ref + rng.normal(0.0, 0.1, ref.shape)
    noisy = visqol(mono(ref, 16_000), mono(deg, 16_000), mode="speech")
    assert 1.0 <= noisy < 5.0


@pytest.mark.skipif(not CLEAN_SPEECH.is_dir(), reason="speech testdata not present")
def test_speech_conformance_value():
    score = visqol(
        CLEAN_SPEECH / "CA01_01.wav",
        CLEAN_SPEECH / "transcoded_CA01_01.wav",
        mode="speech",
    )
    # kConformanceSpeechCA01TranscodedExponential. CA01_01 is < 1 s long, so
    # this pair uses the port's short-duration tolerance (0.08).
    assert score == pytest.approx(3.374505555111911, abs=0.08)


# ── file-path inputs ─────────────────────────────────────────────────────────


@pytest.mark.skipif(not CONFORMANCE.is_dir(), reason="conformance testdata not present")
def test_file_scoring_matches_conformance_value():
    score = visqol(
        str(CONFORMANCE / "strauss48_stereo.wav"),
        CONFORMANCE / "strauss48_stereo_lp35.wav",  # str and PathLike both work
    )
    # C++ reference value; Rust port tolerance is 0.025 for full-length pairs.
    assert score == pytest.approx(1.3888791489130758, abs=0.025)


@pytest.mark.skipif(not CLEAN_SPEECH.is_dir(), reason="speech testdata not present")
def test_mixed_path_and_audio_samples_inputs():
    from audio_samples import read

    path = CLEAN_SPEECH / "CA01_01.wav"
    loaded = read(str(path))
    from_paths = visqol(path, path)
    from_mixed = visqol(path, loaded)
    assert from_paths == pytest.approx(from_mixed, abs=1e-6)


def test_missing_file_raises_oserror(tmp_path):
    with pytest.raises(OSError):
        visqol(tmp_path / "nope_ref.wav", tmp_path / "nope_deg.wav")


# ── argument validation ──────────────────────────────────────────────────────


def test_raw_array_rejected_with_helpful_message():
    x = sine()
    with pytest.raises(TypeError, match="new_mono"):
        visqol(x, x)  # type: ignore[arg-type]


def test_bad_mode_rejected():
    x = mono(sine())
    with pytest.raises(ValueError, match="mode"):
        visqol(x, x, mode="music")  # type: ignore[arg-type]


def test_too_short_signal_raises_visqol_error():
    x = mono(sine(seconds=0.05))  # far below one analysis patch
    with pytest.raises(VisqolError):
        visqol(x, x)


# ── package surface ──────────────────────────────────────────────────────────


def test_public_api_surface():
    assert audio_samples_qoe.__version__
    for name in audio_samples_qoe.__all__:
        assert hasattr(audio_samples_qoe, name), name
    assert issubclass(VisqolError, Exception)


def test_docstrings_present():
    assert "MOS-LQO" in visqol.__doc__
    assert VisqolError.__doc__
    assert audio_samples_qoe.__doc__
