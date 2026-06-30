//! SCOREQ — Speech Contrastive Regression for Quality Assessment.
//!
//! A neural speech-quality metric (Ragano et al., NeurIPS 2024) built on a
//! wav2vec2 encoder. Unlike ViSQOL (pure DSP), SCOREQ runs a trained model via
//! ONNX Runtime. Two operating modes across two domains:
//!
//! - **No-reference (`Nr`)**: predicts a Mean Opinion Score from the test
//!   signal alone.
//! - **Reference (`Ref`)**: returns the Euclidean distance between the test and
//!   reference embeddings (smaller = closer to the reference's quality).
//!
//! The first use of a given model downloads ~378 MB of weights from Zenodo into
//! `~/.cache/scoreq/`. Construct a [`Scoreq`] once and reuse it; loading the
//! model is expensive.
//!
//! # Sample rate
//!
//! SCOREQ operates at 16 kHz. Inputs at any other rate are resampled with
//! `audio_samples`/`rubato`, which differs from upstream's torchaudio kernel by
//! ~0.05–0.09 MOS. **Feed 16 kHz audio for exact agreement with the reference
//! implementation** (16 kHz inputs bypass resampling entirely).

mod model;
mod preprocess;
mod weights;

use audio_samples::{AudioSamples, StandardSample};

use crate::error::{AudioQoEError, AudioQoEResult};
use model::ScoreqSession;

/// Which pre-trained model family to use. Choosing the wrong domain yields
/// meaningless scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreqDomain {
    /// Recorded human speech: codecs, VoIP, telephony, enhancement, restoration.
    Natural,
    /// Machine-generated speech: TTS, voice conversion, generative models.
    Synthetic,
}

/// Operating mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreqMode {
    /// No-reference: predict a MOS from the test signal alone.
    Nr,
    /// Reference-based: Euclidean distance between test and reference embeddings.
    Ref,
}

/// A loaded SCOREQ model. Holds the ONNX session; construct once and reuse.
pub struct Scoreq {
    domain: ScoreqDomain,
    mode: ScoreqMode,
    session: ScoreqSession,
}

impl Scoreq {
    /// Load the model for `domain`/`mode`, downloading weights on first use.
    pub fn new(domain: ScoreqDomain, mode: ScoreqMode) -> AudioQoEResult<Self> {
        let path = weights::resolve_model(domain, mode)?;
        let session = ScoreqSession::load(&path)?;
        Ok(Self {
            domain,
            mode,
            session,
        })
    }

    /// The domain this model was loaded for.
    pub fn domain(&self) -> ScoreqDomain {
        self.domain
    }

    /// The mode this model was loaded for.
    pub fn mode(&self) -> ScoreqMode {
        self.mode
    }

    /// No-reference MOS prediction for `test`. Errors unless loaded in
    /// [`ScoreqMode::Nr`].
    pub fn predict<T: StandardSample>(
        &mut self,
        test: &AudioSamples<'_, T>,
    ) -> AudioQoEResult<f64> {
        if self.mode != ScoreqMode::Nr {
            return Err(AudioQoEError::scoreq(
                "predict() requires Nr mode; use predict_ref() for Ref mode",
            ));
        }
        let samples = preprocess::to_model_input(test)?;
        self.session.predict_mos(&samples)
    }

    /// Reference-based distance between `test` and `reference` (Euclidean
    /// distance of their embeddings). Errors unless loaded in [`ScoreqMode::Ref`].
    pub fn predict_ref<T: StandardSample>(
        &mut self,
        test: &AudioSamples<'_, T>,
        reference: &AudioSamples<'_, T>,
    ) -> AudioQoEResult<f64> {
        if self.mode != ScoreqMode::Ref {
            return Err(AudioQoEError::scoreq(
                "predict_ref() requires Ref mode; use predict() for Nr mode",
            ));
        }
        let test_emb = self.session.embedding(&preprocess::to_model_input(test)?)?;
        let ref_emb = self
            .session
            .embedding(&preprocess::to_model_input(reference)?)?;
        if test_emb.len() != ref_emb.len() {
            return Err(AudioQoEError::scoreq("embedding dimension mismatch"));
        }
        let dist = test_emb
            .iter()
            .zip(&ref_emb)
            .map(|(a, b)| {
                let d = (a - b) as f64;
                d * d
            })
            .sum::<f64>()
            .sqrt();
        Ok(dist)
    }
}
