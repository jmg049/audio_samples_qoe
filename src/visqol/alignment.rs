use audio_samples::{AudioEditing, AudioSamples, fft_alignment_lag};

use crate::error::AudioQoEResult;

/// Globally align `deg` to `ref_audio` by FFT cross-correlation.
///
/// Returns `(aligned_deg, lag_seconds)`.  If the detected lag is zero or
/// exceeds half the reference length the original degraded is returned
/// unchanged with lag = 0.
pub fn globally_align(
    ref_audio: &AudioSamples<'_, f64>,
    deg_audio: AudioSamples<'_, f64>,
) -> AudioQoEResult<(AudioSamples<'static, f64>, f64)> {
    let sr = ref_audio.sample_rate().get() as usize;
    let ref_len = ref_audio.samples_per_channel().get();
    let max_lag = ref_len / 2;

    let lag = fft_alignment_lag(ref_audio, &deg_audio, max_lag).unwrap_or(0);

    if lag == 0 || lag.unsigned_abs() as usize > max_lag {
        return Ok((deg_audio.into_owned(), 0.0));
    }

    let lag_secs = lag as f64 / sr as f64;

    let aligned = if lag < 0 {
        let trim_secs = lag_secs.abs();
        let total_secs = deg_audio.duration_seconds();
        deg_audio.trim(trim_secs, total_secs)?.into_owned()
    } else {
        deg_audio.pad(lag_secs, 0.0, 0.0_f64)?.into_owned()
    };

    Ok((aligned, lag_secs))
}
