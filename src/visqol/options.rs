use std::num::NonZeroU32;

use audio_samples::operations::ResamplingQuality;
use spectrograms::{ErbSpacing, GammatoneParams, SpectrogramParams, StftParams, WindowType, nzu};

/// ViSQOL mode: full audio (48kHz, 32 bands) or speech (16kHz, 21 bands).
#[derive(Clone, Copy, Debug, Default)]
pub enum VisqolMode {
    #[default]
    Audio,
    Speech,
}

pub struct VisqolOptions {
    pub mode: VisqolMode,
    pub resample_quality: ResamplingQuality,
    pub stft_params: SpectrogramParams,
    pub gammatone_params: GammatoneParams,
    /// Speech mode only: scale the NSIM→MOS exponential fit so a perfect
    /// NSIM of 1.0 maps to MOS 5.0 instead of ~4.0. Matches the C++
    /// behaviour when `use_unscaled_speech_mos_mapping` is left off.
    pub scale_speech_mos: bool,
}

impl VisqolOptions {
    pub fn audio() -> Self {
        let sr = 48_000.0_f64;
        let stft = StftParams::new(nzu!(3840), nzu!(960), WindowType::Hanning, true)
            .expect("valid stft params");
        let stft_params = SpectrogramParams::new(stft, sr).expect("valid spectrogram params");
        let gammatone_params = GammatoneParams::new(nzu!(32), 50.0, sr / 2.0)
            .expect("valid gammatone params")
            .with_spacing(ErbSpacing::AppleTr35)
            .with_db_floor(-45.0);
        Self {
            mode: VisqolMode::Audio,
            resample_quality: ResamplingQuality::default(),
            stft_params,
            gammatone_params,
            scale_speech_mos: true,
        }
    }

    pub fn speech() -> Self {
        let sr = 16_000.0_f64;
        let stft = StftParams::new(nzu!(1280), nzu!(320), WindowType::Hanning, true)
            .expect("valid stft params");
        let stft_params = SpectrogramParams::new(stft, sr).expect("valid spectrogram params");
        let gammatone_params = GammatoneParams::new(nzu!(21), 50.0, 8_000.0)
            .expect("valid gammatone params")
            .with_spacing(ErbSpacing::AppleTr35)
            .with_db_floor(-45.0);
        Self {
            mode: VisqolMode::Speech,
            resample_quality: ResamplingQuality::default(),
            stft_params,
            gammatone_params,
            scale_speech_mos: true,
        }
    }

    pub fn sample_rate(&self) -> NonZeroU32 {
        match self.mode {
            VisqolMode::Audio => NonZeroU32::new(48_000).unwrap(),
            VisqolMode::Speech => NonZeroU32::new(16_000).unwrap(),
        }
    }
}
