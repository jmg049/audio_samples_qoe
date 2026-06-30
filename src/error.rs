use audio_samples_io::AudioIOError;
use thiserror::Error;
pub type AudioQoEResult<T> = Result<T, AudioQoEError>;


#[derive(Error, Debug)]
pub enum AudioQoEError {
    #[error("IO error: {0}")]
    IOError(#[from] AudioIOError),
    #[error("Audio sample error: {0}")]
    AudioSamplesError(#[from] audio_samples::AudioSampleError),
    #[error("ViSQOL error: {0}")]
    VisqolError(String),
    #[cfg(feature = "scoreq")]
    #[error("SCOREQ error: {0}")]
    ScoreqError(String),
}

impl AudioQoEError {
    pub(crate) fn visqol(msg: impl Into<String>) -> Self {
        AudioQoEError::VisqolError(msg.into())
    }

    #[cfg(feature = "scoreq")]
    pub(crate) fn scoreq(msg: impl Into<String>) -> Self {
        AudioQoEError::ScoreqError(msg.into())
    }
}