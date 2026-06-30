//! Thin wrapper over an `ort` session running a SCOREQ ONNX model.
//!
//! The graph takes a `[1, samples]` f32 tensor. NR models emit a `[1, 1]` MOS
//! scalar; REF models emit a `[1, D]` L2-normalized embedding.

use std::path::Path;

use ort::session::Session;
use ort::value::Tensor;

use crate::error::{AudioQoEError, AudioQoEResult};

pub(crate) struct ScoreqSession {
    session: Session,
    input_name: String,
}

impl ScoreqSession {
    pub(crate) fn load(path: &Path) -> AudioQoEResult<Self> {
        let session = Session::builder()
            .map_err(err)?
            .commit_from_file(path)
            .map_err(err)?;
        let input_name = session
            .inputs()
            .first()
            .map(|i| i.name().to_string())
            .ok_or_else(|| AudioQoEError::scoreq("model has no inputs"))?;
        Ok(Self {
            session,
            input_name,
        })
    }

    /// Run the model on a preprocessed sample buffer, returning the flat output.
    fn run(&mut self, samples: &[f32]) -> AudioQoEResult<Vec<f32>> {
        let input = Tensor::from_array(([1usize, samples.len()], samples.to_vec())).map_err(err)?;
        let outputs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => input])
            .map_err(err)?;
        let (_shape, data) = outputs[0].try_extract_tensor::<f32>().map_err(err)?;
        Ok(data.to_vec())
    }

    /// NR mode: scalar MOS prediction.
    pub(crate) fn predict_mos(&mut self, samples: &[f32]) -> AudioQoEResult<f64> {
        let out = self.run(samples)?;
        out.first()
            .map(|&v| v as f64)
            .ok_or_else(|| AudioQoEError::scoreq("empty model output"))
    }

    /// REF mode: the L2-normalized embedding vector.
    pub(crate) fn embedding(&mut self, samples: &[f32]) -> AudioQoEResult<Vec<f32>> {
        self.run(samples)
    }
}

fn err(e: impl std::fmt::Display) -> AudioQoEError {
    AudioQoEError::scoreq(e.to_string())
}
