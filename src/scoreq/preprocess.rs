//! Audio preprocessing for SCOREQ: arbitrary input → 16 kHz mono f32,
//! zero-padded to a multiple of the wav2vec2 CNN stride.
//!
//! Mirrors upstream `load_processing` + `dynamic_pad`.
//!
//! Resampler fidelity: upstream resamples with torchaudio's `sinc_interp_hann`
//! kernel; we use `rubato` (`ResamplingQuality::High`, via `audio_samples`).
//! Measured against a torchaudio oracle on real 48 kHz speech, this costs
//! ~0.05–0.09 MOS versus upstream. Inputs already at 16 kHz bypass resampling
//! and therefore match upstream exactly — feed 16 kHz audio when upstream
//! conformance matters.

use audio_samples::operations::ResamplingQuality;
use audio_samples::operations::types::MonoConversionMethod;
use audio_samples::{
    AudioChannelOps, AudioData, AudioProcessing, AudioSamples, AudioTypeConversion, StandardSample,
};
use core::num::NonZeroU32;

use crate::error::AudioQoEResult;

/// wav2vec2 feature-extractor total stride; input length must be a multiple.
const PADDING_MULTIPLE: usize = 320;
/// SCOREQ operates at 16 kHz.
const TARGET_SR: u32 = 16_000;

/// Produce the model input vector: 16 kHz mono f32, padded to a multiple of 320.
pub(crate) fn to_model_input<T: StandardSample>(
    audio: &AudioSamples<'_, T>,
) -> AudioQoEResult<Vec<f32>> {
    let mono = audio.as_f32().to_mono(MonoConversionMethod::Average)?;
    let mono = if mono.sample_rate().get() == TARGET_SR {
        mono
    } else {
        let sr = NonZeroU32::new(TARGET_SR).expect("16000 is non-zero");
        mono.resample(sr, ResamplingQuality::High)?
    };

    let mut samples = match &mono.data {
        AudioData::Mono(m) => m.as_view().to_vec(),
        AudioData::Multi(_) => unreachable!("to_mono guarantees a mono signal"),
    };

    let remainder = samples.len() % PADDING_MULTIPLE;
    if remainder != 0 {
        samples.resize(samples.len() + (PADDING_MULTIPLE - remainder), 0.0);
    }
    Ok(samples)
}
