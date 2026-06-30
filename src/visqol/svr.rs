// libsvm nu-SVR prediction for ViSQOL audio mode.
// RBF kernel: K(x,y) = exp(-gamma * ||x-y||^2)
// Model: 317 support vectors, gamma=0.01, rho=-3.6816207123583506.

use crate::error::AudioQoEResult;

/// One support vector with its alpha coefficient.
struct SupportVector {
    alpha: f64,
    features: Vec<f64>,
}

pub struct VisqolSvr {
    svs: Vec<SupportVector>,
    gamma: f64,
    rho: f64,
    n_features: usize,
}

impl VisqolSvr {
    /// Parse the libsvm model text format.
    pub fn from_model_str(text: &str) -> AudioQoEResult<Self> {
        let mut gamma = 0.01_f64;
        let mut rho = 0.0_f64;
        let mut n_features = 0usize;
        let mut svs: Vec<SupportVector> = Vec::new();
        let mut in_sv_section = false;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == "SV" {
                in_sv_section = true;
                continue;
            }
            if !in_sv_section {
                if let Some(rest) = line.strip_prefix("gamma ") {
                    gamma = rest.trim().parse().map_err(|e| {
                        crate::error::AudioQoEError::visqol(format!("bad gamma: {e}"))
                    })?;
                } else if let Some(rest) = line.strip_prefix("rho ") {
                    rho = rest.trim().parse().map_err(|e| {
                        crate::error::AudioQoEError::visqol(format!("bad rho: {e}"))
                    })?;
                }
                continue;
            }

            // SV line: alpha feat1:val feat2:val …
            let mut parts = line.split_whitespace();
            let alpha: f64 = parts
                .next()
                .ok_or_else(|| crate::error::AudioQoEError::visqol("empty SV line"))?
                .parse()
                .map_err(|e| crate::error::AudioQoEError::visqol(format!("bad alpha: {e}")))?;

            let mut features: Vec<(usize, f64)> = Vec::new();
            for tok in parts {
                if let Some((idx_s, val_s)) = tok.split_once(':') {
                    let idx: usize = idx_s.parse().map_err(|e| {
                        crate::error::AudioQoEError::visqol(format!("bad feature index: {e}"))
                    })?;
                    let val: f64 = val_s.parse().map_err(|e| {
                        crate::error::AudioQoEError::visqol(format!("bad feature value: {e}"))
                    })?;
                    features.push((idx, val));
                    n_features = n_features.max(idx);
                }
            }

            // Dense feature vector (1-indexed → 0-indexed).
            let mut dense = vec![0.0_f64; n_features];
            for (idx, val) in features {
                if idx > 0 {
                    let i = idx - 1;
                    if i >= dense.len() {
                        dense.resize(i + 1, 0.0);
                    }
                    dense[i] = val;
                }
            }
            svs.push(SupportVector { alpha, features: dense });
        }

        if svs.is_empty() {
            return Err(crate::error::AudioQoEError::visqol("no support vectors found"));
        }

        Ok(Self { svs, gamma, rho, n_features })
    }

    /// Predict MOS-LQO from a 32-element fvnsim vector, clipped to [1.0, 5.0].
    ///
    /// Errors if `fvnsim`'s length does not match the model's feature
    /// dimensionality, which would otherwise silently truncate the RBF distance.
    pub fn predict(&self, fvnsim: &[f64]) -> AudioQoEResult<f64> {
        if fvnsim.len() != self.n_features {
            return Err(crate::error::AudioQoEError::visqol(format!(
                "fvnsim length {} does not match model feature count {}",
                fvnsim.len(),
                self.n_features
            )));
        }

        let decision: f64 = self
            .svs
            .iter()
            .map(|sv| {
                let dist_sq: f64 = fvnsim
                    .iter()
                    .zip(sv.features.iter())
                    .map(|(&x, &s)| (x - s).powi(2))
                    .sum();
                sv.alpha * f64::exp(-self.gamma * dist_sq)
            })
            .sum::<f64>()
            - self.rho;

        Ok(decision.clamp(1.0, 5.0))
    }
}
