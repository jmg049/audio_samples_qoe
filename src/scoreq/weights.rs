//! Model-weight resolution: locate the ONNX file in the local cache
//! (`~/.cache/scoreq/onnx-models/`, mirroring the upstream Python package),
//! downloading it from Zenodo on first use.

use std::path::{Path, PathBuf};

use crate::error::{AudioQoEError, AudioQoEResult};
use crate::scoreq::{ScoreqDomain, ScoreqMode};

const ZENODO_BASE: &str = "https://zenodo.org/records/15739280/files";

/// The released ONNX filename for a given domain/mode combination.
pub(crate) fn onnx_filename(domain: ScoreqDomain, mode: ScoreqMode) -> &'static str {
    match (domain, mode) {
        (ScoreqDomain::Natural, ScoreqMode::Nr) => "adapt_nr_telephone.onnx",
        (ScoreqDomain::Natural, ScoreqMode::Ref) => "fixed_nmr_telephone.onnx",
        (ScoreqDomain::Synthetic, ScoreqMode::Nr) => "adapt_nr_synthetic.onnx",
        (ScoreqDomain::Synthetic, ScoreqMode::Ref) => "fixed_nmr_synthetic.onnx",
    }
}

fn cache_dir() -> AudioQoEResult<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| AudioQoEError::scoreq("HOME environment variable is not set"))?;
    Ok(PathBuf::from(home).join(".cache/scoreq/onnx-models"))
}

/// Resolve the local path to a model, downloading it on first use.
pub(crate) fn resolve_model(domain: ScoreqDomain, mode: ScoreqMode) -> AudioQoEResult<PathBuf> {
    let filename = onnx_filename(domain, mode);
    let path = cache_dir()?.join(filename);
    if !path.exists() {
        let dir = path.parent().expect("cache path has a parent");
        std::fs::create_dir_all(dir).map_err(|e| AudioQoEError::scoreq(e.to_string()))?;
        download(&format!("{ZENODO_BASE}/{filename}"), &path)?;
    }
    Ok(path)
}

/// Stream `url` to `dest` via a temporary `.part` file, renamed on success so a
/// partial download is never mistaken for a complete cache entry.
fn download(url: &str, dest: &Path) -> AudioQoEResult<()> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| AudioQoEError::scoreq(format!("downloading {url}: {e}")))?;
    let tmp = dest.with_extension("part");
    let mut file =
        std::fs::File::create(&tmp).map_err(|e| AudioQoEError::scoreq(e.to_string()))?;
    std::io::copy(&mut resp.into_body().into_reader(), &mut file)
        .map_err(|e| AudioQoEError::scoreq(format!("writing {}: {e}", tmp.display())))?;
    std::fs::rename(&tmp, dest).map_err(|e| AudioQoEError::scoreq(e.to_string()))?;
    Ok(())
}
