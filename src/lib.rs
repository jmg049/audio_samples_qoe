mod error;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "scoreq")]
mod scoreq;
mod utils;
mod visqol;

pub use error::{AudioQoEError, AudioQoEResult};
#[cfg(feature = "scoreq")]
pub use scoreq::{Scoreq, ScoreqDomain, ScoreqMode};
pub use visqol::{VisqolMode, VisqolOptions, visqol, visqol_with_source};

use audio_samples::{AudioSamples, AudioTypeConversion, StandardSample};

pub trait AudioSamplesQoE: AudioTypeConversion
where
    Self::Sample: StandardSample,
{
    fn visqol(&self, degraded: &Self) -> AudioQoEResult<f64>;
    fn visqol_with_options(&self, degraded: &Self, opts: &VisqolOptions) -> AudioQoEResult<f64>;
}

impl<T> AudioSamplesQoE for AudioSamples<'_, T>
where
    T: StandardSample,
{
    fn visqol(&self, degraded: &Self) -> AudioQoEResult<f64> {
        let opts = VisqolOptions::audio();
        visqol::visqol(self, degraded.clone(), &opts)
    }

    fn visqol_with_options(&self, degraded: &Self, opts: &VisqolOptions) -> AudioQoEResult<f64> {
        visqol::visqol(self, degraded.clone(), opts)
    }
}
