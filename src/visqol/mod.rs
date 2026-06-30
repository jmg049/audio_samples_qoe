mod alignment;
mod nsim;
mod options;
mod patches;
mod spectrogram_ops;
mod svr;
mod vad;

pub use options::{VisqolMode, VisqolOptions};

// Re-export the spectrogram-source abstraction so callers can build custom
// representations without a separate `spectrograms` import.
pub use spectrograms::{GammatoneSource, SpectrogramSource};

use std::sync::OnceLock;

use audio_samples::{AudioChannelOps, AudioData, AudioProcessing, AudioSamples, AudioTypeConversion, StandardSample};
use audio_samples::operations::types::MonoConversionMethod;
use ndarray::Array1;

use crate::error::AudioQoEResult;

/// Default patch size in frames (audio mode).
const PATCH_SIZE_AUDIO: usize = 30;
/// Default patch size in frames (speech mode).
const PATCH_SIZE_SPEECH: usize = 20;
/// DTW search window radius in units of patches.
const SEARCH_WINDOW_RADIUS: usize = 60;

static AUDIO_SVR: OnceLock<svr::VisqolSvr> = OnceLock::new();

fn audio_svr() -> &'static svr::VisqolSvr {
    AUDIO_SVR.get_or_init(|| {
        let model_text = include_str!("model/libsvm_nu_svr_model.txt");
        svr::VisqolSvr::from_model_str(model_text).expect("built-in SVR model is valid")
    })
}

/// Compute the ViSQOL MOS-LQO score between a reference and degraded signal.
///
/// Input audio may be any sample type, any channel count, and any sample rate.
/// Internally normalised to f64 mono (channel average) at the rate in `opts`.
///
/// This uses the time-domain IIR gammatone spectrogram the ViSQOL SVR model
/// was trained on — the only representation that yields conformant MOS-LQO
/// scores. To run the same pipeline over a different spectrogram (mel, ERB,
/// CQT, or your own), build a [`spectrograms::SpectrogramSource`] and call
/// [`visqol_with_source`] instead.
pub fn visqol<T>(
    ref_audio: &AudioSamples<'_, T>,
    deg_audio: AudioSamples<'_, T>,
    opts: &VisqolOptions,
) -> AudioQoEResult<f64>
where
    T: StandardSample,
{
    let source = default_gammatone_source(ref_audio, opts)?;
    visqol_with_source(ref_audio, deg_audio, opts, source)
}

/// Compute a ViSQOL-style score using a caller-supplied spectrogram source.
///
/// The pipeline (alignment, SPL scaling, per-frame floor, patch matching, NSIM,
/// NSIM→MOS mapping) is identical to [`visqol`]; only the time-frequency
/// representation changes. `source` defines the operating sample rate, framing,
/// and band layout — both signals are resampled to `source.sample_rate()` and
/// passed through the same source.
///
/// # Conformance
///
/// Only the IIR gammatone representation produces true ViSQOL MOS-LQO values;
/// the audio-mode SVR expects exactly its 32-band fvnsim feature vector and
/// will error if `source.n_bands()` differs. Other representations are useful
/// for research and comparison but are **not** calibrated ViSQOL scores.
///
/// # Errors
///
/// Returns an error if the source's sample rate is invalid, if audio
/// preparation or the spectrogram computation fails, if the reference is too
/// short to yield any patches, or (audio mode) if the band count does not match
/// the SVR model.
pub fn visqol_with_source<T, S>(
    ref_audio: &AudioSamples<'_, T>,
    deg_audio: AudioSamples<'_, T>,
    opts: &VisqolOptions,
    mut source: S,
) -> AudioQoEResult<f64>
where
    T: StandardSample,
    S: SpectrogramSource<f64>,
{
    // The source dictates the operating rate; both signals are resampled to it.
    let rate = std::num::NonZeroU32::new(source.sample_rate().round() as u32).ok_or_else(|| {
        crate::error::AudioQoEError::visqol("spectrogram source sample rate must be > 0")
    })?;
    let ref_f64 = prepare_audio(ref_audio, rate, opts)?;
    let deg_f64 = prepare_audio(&deg_audio, rate, opts)?;

    // 1. Global alignment.
    let (deg_f64, _lag_secs) = alignment::globally_align(&ref_f64, deg_f64)?;

    // 2. Scale degraded to reference sound-pressure level.
    let ref_spl = crate::utils::sound_pressure_level(&ref_f64);
    let deg_f64 = crate::utils::scale_to_spl(deg_f64, ref_spl);

    // 3. Spectrogram via the pluggable source (dB output expected by step 4).
    let ref_samples_f64 = mono_samples(&ref_f64);
    let deg_samples_f64 = mono_samples(&deg_f64);

    let mut ref_data = source
        .compute_matrix(&ref_samples_f64)
        .map_err(|e| crate::error::AudioQoEError::visqol(e.to_string()))?;
    let mut deg_data = source
        .compute_matrix(&deg_samples_f64)
        .map_err(|e| crate::error::AudioQoEError::visqol(e.to_string()))?;

    // 4. Per-frame floor + global floor subtraction.
    spectrogram_ops::prepare_for_comparison(&mut ref_data, &mut deg_data);

    // 5. Patch matching.
    let patch_size = match opts.mode {
        VisqolMode::Audio => PATCH_SIZE_AUDIO,
        VisqolMode::Speech => PATCH_SIZE_SPEECH,
    };

    // Speech mode gates reference patches on voice activity, so silence in
    // the reference is excluded from the comparison.
    let hop_size_samples = (source.hop_seconds() * source.sample_rate()).round() as usize;
    let ref_indices = match opts.mode {
        VisqolMode::Audio => patches::ref_patch_indices(ref_data.ncols(), patch_size),
        VisqolMode::Speech => vad::vad_ref_patch_indices(
            &ref_samples_f64,
            ref_data.ncols(),
            patch_size,
            hop_size_samples,
        ),
    };
    if ref_indices.is_empty() {
        return Err(crate::error::AudioQoEError::visqol(
            "reference spectrogram is too short to extract any patches",
        ));
    }

    let frame_duration = source.hop_seconds();

    let patch_sims = patches::find_best_patches(
        ref_data.view(),
        deg_data.view(),
        &ref_indices,
        frame_duration,
        patch_size,
        SEARCH_WINDOW_RADIUS,
    )?;

    // 6. Per-band similarity statistics.
    let n_bands = ref_data.nrows();
    let n_patches = patch_sims.len();

    let fvnsim = per_band_mean(&patch_sims, n_bands, n_patches, |p| &p.freq_band_means);
    let _fvnsim10 = per_band_quantile(&patch_sims, n_bands, 0.10);
    let _fstdnsim = per_band_pooled_stddev(&patch_sims, &fvnsim, n_bands, frame_duration);
    let _fvdegenergy = per_band_mean(&patch_sims, n_bands, n_patches, |p| &p.freq_band_deg_energy);

    // 7. NSIM → MOS mapping: SVR for audio mode, exponential fit for speech.
    let vnsim = fvnsim.mean().unwrap_or(0.0);
    let mut mos = match opts.mode {
        VisqolMode::Audio => audio_svr().predict(&fvnsim.to_vec())?,
        VisqolMode::Speech => speech_mos(vnsim, opts.scale_speech_mos),
    };

    // 8. Clamp dissimilar signals.
    if vnsim < 0.15 {
        mos = 1.0;
    }

    Ok(mos)
}

/// Build the conformant gammatone source for `opts`, mirroring the C++
/// reference: audio mode runs at the calibrated 48 kHz with the option template;
/// speech mode never resamples the reference, running at its native rate with an
/// 80 ms window, 25% hop, and the bank capped at 8 kHz.
fn default_gammatone_source<T: StandardSample>(
    ref_audio: &AudioSamples<'_, T>,
    opts: &VisqolOptions,
) -> AudioQoEResult<GammatoneSource> {
    let (stft_params, gammatone_params, rate) = match opts.mode {
        VisqolMode::Audio => (
            opts.stft_params.clone(),
            opts.gammatone_params,
            opts.sample_rate(),
        ),
        VisqolMode::Speech => {
            let rate = ref_audio.sample_rate();
            let (stft_params, gammatone_params) = speech_params(rate, &opts.gammatone_params)?;
            (stft_params, gammatone_params, rate)
        }
    };
    Ok(GammatoneSource::new(
        f64::from(rate.get()),
        stft_params.stft().n_fft(),
        stft_params.stft().hop_size(),
        gammatone_params,
    ))
}

/// Speech-mode MOS from mean NSIM via the exponential fit over the TCD-VOIP
/// dataset. Matches C++ SpeechSimilarityToQualityMapper::PredictQuality.
fn speech_mos(nsim_mean: f64, scale_to_max_mos: bool) -> f64 {
    const FIT_PARAMETER_A: f64 = -262.847_869;
    const FIT_PARAMETER_B: f64 = 0.015_430_252_5;
    const FIT_PARAMETER_X0: f64 = -361.063_949;
    const FIT_SCALE: f64 = 1.245_063;

    let mos = FIT_PARAMETER_A + (FIT_PARAMETER_B * (nsim_mean - FIT_PARAMETER_X0)).exp();
    let scale = if scale_to_max_mos { FIT_SCALE } else { 1.0 };
    (mos * scale).clamp(1.0, 5.0)
}

/// Mix down to f64 mono (channel average) and resample to `target_rate`.
fn prepare_audio<T: StandardSample>(
    audio: &AudioSamples<'_, T>,
    target_rate: std::num::NonZeroU32,
    opts: &VisqolOptions,
) -> AudioQoEResult<AudioSamples<'static, f64>> {
    let mono = audio.as_f64().to_mono(MonoConversionMethod::Average)?;
    if mono.sample_rate() == target_rate {
        return Ok(mono);
    }
    Ok(mono.resample(target_rate, opts.resample_quality)?)
}

/// Speech-mode spectrogram parameters for an arbitrary operating rate,
/// mirroring the C++ AnalysisWindow (80 ms window, 25% hop, truncating
/// size casts) and GammatoneSpectrogramBuilder's 8 kHz speech-mode cap.
/// Band count, minimum frequency, and spacing come from the option template.
fn speech_params(
    rate: std::num::NonZeroU32,
    template: &spectrograms::GammatoneParams,
) -> AudioQoEResult<(spectrograms::SpectrogramParams, spectrograms::GammatoneParams)> {
    use spectrograms::{GammatoneParams, SpectrogramParams, StftParams, WindowType};

    let err = |e: &dyn std::fmt::Display| crate::error::AudioQoEError::visqol(e.to_string());
    let sr = f64::from(rate.get());
    let window = (sr * 0.08) as usize;
    let hop = (window as f64 * 0.25) as usize;
    let (window, hop) = match (std::num::NonZeroUsize::new(window), std::num::NonZeroUsize::new(hop)) {
        (Some(w), Some(h)) => (w, h),
        _ => {
            return Err(crate::error::AudioQoEError::visqol(format!(
                "sample rate {sr} Hz is too low for the 80 ms speech analysis window"
            )));
        }
    };
    let stft = StftParams::new(window, hop, WindowType::Hanning, true).map_err(|e| err(&e))?;
    let stft_params = SpectrogramParams::new(stft, sr).map_err(|e| err(&e))?;
    let gammatone_params =
        GammatoneParams::new(template.n_filters(), template.f_min(), template.f_max().min(sr / 2.0))
            .map_err(|e| err(&e))?
            .with_spacing(template.spacing())
            .with_db_floor(-45.0);
    Ok((stft_params, gammatone_params))
}

/// Extract samples from a mono f64 AudioSamples as a plain Vec.
/// Panics on non-mono input — callers must run `prepare_audio` first.
fn mono_samples(audio: &AudioSamples<'_, f64>) -> Vec<f64> {
    match &audio.data {
        AudioData::Mono(m) => m.as_view().to_vec(),
        AudioData::Multi(_) => panic!("expected mono audio — call prepare_audio first"),
    }
}

fn per_band_mean<'a, F>(
    patches: &'a [nsim::PatchSim],
    n_bands: usize,
    n_patches: usize,
    field: F,
) -> Array1<f64>
where
    F: Fn(&'a nsim::PatchSim) -> &'a Array1<f64>,
{
    let mut sum = Array1::<f64>::zeros(n_bands);
    for p in patches {
        sum += field(p);
    }
    sum / n_patches as f64
}

fn per_band_quantile(patches: &[nsim::PatchSim], n_bands: usize, q: f64) -> Array1<f64> {
    let mut result = Array1::<f64>::zeros(n_bands);
    for b in 0..n_bands {
        let mut vals: Vec<f64> = patches.iter().map(|p| p.freq_band_means[b]).collect();
        vals.sort_by(|a, c| a.partial_cmp(c).unwrap());
        let k = ((vals.len() as f64 * q).ceil() as usize).max(1);
        result[b] = vals[..k].iter().sum::<f64>() / k as f64;
    }
    result
}

fn per_band_pooled_stddev(
    patches: &[nsim::PatchSim],
    global_mean: &Array1<f64>,
    n_bands: usize,
    frame_duration: f64,
) -> Array1<f64> {
    let mut contrib = Array1::<f64>::zeros(n_bands);
    let mut total_frames = 0i64;

    for p in patches {
        let dur = p.ref_patch_end_time - p.ref_patch_start_time;
        let fc = (dur / frame_duration).ceil() as i64;
        total_frames += fc;
        for b in 0..n_bands {
            let s = p.freq_band_stddevs[b];
            let m = p.freq_band_means[b];
            contrib[b] += (fc - 1) as f64 * s * s + fc as f64 * m * m;
        }
    }

    if total_frames <= 1 {
        return Array1::zeros(n_bands);
    }

    let gm_sq = global_mean.mapv(|m| m * m) * total_frames as f64;
    let mut var = (contrib - gm_sq) / (total_frames - 1) as f64;
    var.mapv_inplace(|v| if v < 0.0 { 0.0 } else { v.sqrt() });
    var
}
