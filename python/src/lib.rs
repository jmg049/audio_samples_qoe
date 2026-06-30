//! Python bindings for the `audio_samples_qoe` ViSQOL implementation.
//!
//! Exposed as the private extension module `audio_samples_qoe._native`. The
//! public, documented, typed API lives in the `audio_samples_qoe` Python
//! package next to this crate — it dispatches paths and
//! `audio_samples.AudioSamples` objects down to `_score`.

use std::num::NonZeroU32;
use std::path::PathBuf;

use audio_samples::{AudioData, AudioSamples};
use audio_samples_qoe::{AudioQoEError, AudioSamplesQoE, VisqolOptions};
use ndarray::{ArrayViewD, Ix1, Ix2};
use numpy::{AllowTypeChange, IntoPyArray, PyArrayLikeDyn};
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyIOError, PyValueError};
use pyo3::prelude::*;

create_exception!(
    _native,
    VisqolError,
    PyException,
    "Raised when the ViSQOL computation itself fails.\n\n\
     This covers errors inside the metric pipeline — e.g. signals that are\n\
     too short to extract a single analysis patch, resampling failures, or\n\
     alignment failures. Invalid arguments raise ``ValueError`` or\n\
     ``TypeError`` instead, and unreadable files raise ``OSError``."
);

/// Map a core-library error onto the appropriate Python exception type.
fn qoe_err(e: AudioQoEError) -> PyErr {
    match e {
        AudioQoEError::IOError(io) => PyIOError::new_err(io.to_string()),
        other => VisqolError::new_err(other.to_string()),
    }
}

fn options_for_mode(mode: &str) -> PyResult<VisqolOptions> {
    match mode {
        "audio" => Ok(VisqolOptions::audio()),
        "speech" => Ok(VisqolOptions::speech()),
        other => Err(PyValueError::new_err(format!(
            "mode must be 'audio' or 'speech', got {other:?}"
        ))),
    }
}

/// Convert a NumPy view into an owned `AudioSamples` buffer.
///
/// Accepts 1-D (mono) or 2-D `(channels, samples)` arrays; anything else is a
/// `ValueError`. `name` identifies the offending argument in error messages.
fn to_audio(
    arr: ArrayViewD<'_, f64>,
    sample_rate: u32,
    name: &str,
) -> PyResult<AudioSamples<'static, f64>> {
    let rate = NonZeroU32::new(sample_rate).ok_or_else(|| {
        PyValueError::new_err(format!("{name}: sample rate must be a positive integer, got 0"))
    })?;
    let data = match arr.ndim() {
        1 => {
            let mono = arr
                .to_owned()
                .into_dimensionality::<Ix1>()
                .expect("ndim checked above");
            AudioData::new_mono(mono)
        }
        2 => {
            let multi = arr
                .to_owned()
                .into_dimensionality::<Ix2>()
                .expect("ndim checked above");
            AudioData::new_multi(multi)
        }
        n => {
            return Err(PyValueError::new_err(format!(
                "{name}: expected a 1-D mono array or a 2-D (channels, samples) array, \
                 got a {n}-D array"
            )));
        }
    };
    let data = data.map_err(|e| PyValueError::new_err(format!("{name}: {e}")))?;
    Ok(AudioSamples::new(data, rate))
}

/// Score two raw sample buffers. Private — use ``audio_samples_qoe.visqol``.
///
/// ``reference`` and ``degraded`` are float64 arrays, 1-D mono or 2-D
/// ``(channels, samples)``, with their sample rates in Hz. ``mode`` is
/// ``"audio"`` or ``"speech"``. Returns the MOS-LQO score in [1.0, 5.0].
/// The GIL is released while the score is computed.
#[pyfunction]
#[pyo3(signature = (reference, reference_rate, degraded, degraded_rate, mode))]
fn _score(
    py: Python<'_>,
    reference: PyArrayLikeDyn<'_, f64, AllowTypeChange>,
    reference_rate: u32,
    degraded: PyArrayLikeDyn<'_, f64, AllowTypeChange>,
    degraded_rate: u32,
    mode: &str,
) -> PyResult<f64> {
    let opts = options_for_mode(mode)?;
    let ref_audio = to_audio(reference.as_array(), reference_rate, "reference")?;
    let deg_audio = to_audio(degraded.as_array(), degraded_rate, "degraded")?;
    py.detach(|| ref_audio.visqol_with_options(&deg_audio, &opts))
        .map_err(qoe_err)
}

/// Decode an audio file (WAV/FLAC) with the native Rust reader. Private —
/// used by ``audio_samples_qoe.visqol`` to handle path arguments.
///
/// Returns ``(samples, sample_rate)`` where ``samples`` is a float64 array,
/// 1-D for mono or 2-D ``(channels, samples)``, normalised to [-1, 1].
/// The GIL is released while the file is decoded.
#[pyfunction]
fn _read(py: Python<'_>, path: PathBuf) -> PyResult<(Py<PyAny>, u32)> {
    let audio = py
        .detach(|| audio_samples_io::read::<_, f64>(&path))
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    let rate = audio.sample_rate().get();
    let array = match audio.data {
        AudioData::Mono(_) => {
            let mono = audio.data.into_mono_data().expect("matched mono above");
            mono.into_pyarray(py).into_any().unbind()
        }
        AudioData::Multi(_) => {
            let multi = audio.data.into_multi_data().expect("matched multi above");
            multi.into_pyarray(py).into_any().unbind()
        }
    };
    Ok((array, rate))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("VisqolError", m.py().get_type::<VisqolError>())?;
    m.add_function(wrap_pyfunction!(_score, m)?)?;
    m.add_function(wrap_pyfunction!(_read, m)?)?;
    Ok(())
}
