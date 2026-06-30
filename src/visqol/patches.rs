// Patch creation and DTW-based patch matching.
// Ported from ViSQOL's ImagePatchCreator + ComparisonPatchesSelector.

use ndarray::{Array2, ArrayView2, s};

use super::nsim::{self, PatchSim};
use crate::error::AudioQoEResult;

/// Indices (column starts) of reference patches in the spectrogram.
/// Matches C++ ImagePatchCreator::CreateRefPatchIndices.
pub fn ref_patch_indices(n_frames: usize, patch_size: usize) -> Vec<usize> {
    let init = patch_size / 2;
    if n_frames < patch_size + init {
        return vec![];
    }
    let max_index = n_frames - patch_size;
    let mut indices = Vec::new();
    let mut i = init;
    while i < max_index {
        indices.push(i.saturating_sub(1));
        i += patch_size;
    }
    indices
}

/// Extract a patch from `spectrogram_data` starting at column `start`.
/// Zero-pads columns that fall outside the data range.
pub fn extract_patch(data: ArrayView2<f64>, start: usize, patch_size: usize) -> Array2<f64> {
    let n_bands = data.nrows();
    let n_frames = data.ncols();
    let mut patch = Array2::<f64>::zeros((n_bands, patch_size));
    for (dst_col, src_col) in (start..).take(patch_size).enumerate() {
        if src_col < n_frames {
            patch.slice_mut(s![.., dst_col]).assign(&data.column(src_col));
        }
        // else: column stays zero (silence padding)
    }
    patch
}

/// DTW patch matching: for each reference patch find the best-aligned degraded patch.
/// `search_window_radius` is in units of patches (C++ default = 60).
/// Returns one `PatchSim` per reference patch, with times filled in.
pub fn find_best_patches(
    ref_data: ArrayView2<f64>,
    deg_data: ArrayView2<f64>,
    patch_indices: &[usize],
    frame_duration: f64,
    patch_size: usize,
    search_window_radius: usize,
) -> AudioQoEResult<Vec<PatchSim>> {
    if patch_indices.is_empty() {
        return Err(crate::error::AudioQoEError::visqol(
            "no reference patches to match",
        ));
    }

    let n_frames_deg = deg_data.ncols();
    let search_window = search_window_radius * patch_size;
    let num_patches = clamp_patches(patch_indices, n_frames_deg, patch_size);

    if num_patches == 0 {
        return Err(crate::error::AudioQoEError::visqol(
            "degraded file is too short to score any reference patch",
        ));
    }

    let patch_indices = &patch_indices[..num_patches];
    let patch_dur = patch_size as f64 * frame_duration;

    // Pre-build all degraded patches (one per possible start column).
    let deg_patches: Vec<Array2<f64>> = (0..n_frames_deg)
        .map(|start| extract_patch(deg_data, start, patch_size))
        .collect();

    // DP tables: cumulative similarity and backpointer.
    let mut dp = vec![vec![f64::NEG_INFINITY; n_frames_deg]; num_patches];
    let mut back: Vec<Vec<i64>> = vec![vec![-1; n_frames_deg]; num_patches];

    for (pi, &ref_start) in patch_indices.iter().enumerate() {
        let ref_patch = extract_patch(ref_data, ref_start, patch_size);
        let lo = (ref_start as i64 - search_window as i64).max(0) as usize;
        let hi = (ref_start + search_window).min(n_frames_deg.saturating_sub(1));

        for di in lo..=hi {
            let mut sim = nsim::measure(ref_patch.view(), deg_patches[di].view()).similarity;
            let mut best_prev = -1i64;

            if pi > 0 {
                let prev_lo = (patch_indices[pi - 1] as i64 - search_window as i64).max(0) as usize;
                let mut highest = f64::NEG_INFINITY;
                // Backtrack: find best previous di' < di
                for prev_di in prev_lo..di {
                    if dp[pi - 1][prev_di] > highest {
                        highest = dp[pi - 1][prev_di];
                        best_prev = prev_di as i64;
                    }
                }
                if highest > f64::NEG_INFINITY {
                    sim += highest;
                }
                // Packet-loss: propagate prev score without a new match.
                if pi > 0 && dp[pi - 1][di] > sim {
                    sim = dp[pi - 1][di];
                    best_prev = di as i64;
                }
            }

            dp[pi][di] = sim;
            back[pi][di] = best_prev;
        }
    }

    // Find best final offset.
    let last_pi = num_patches - 1;
    let last_ref_start = patch_indices[last_pi];
    let last_lo = (last_ref_start as i64 - search_window as i64).max(0) as usize;
    let last_hi = (last_ref_start + search_window).min(n_frames_deg.saturating_sub(1));

    let mut best_score = f64::NEG_INFINITY;
    let mut last_di = last_lo;
    for di in last_lo..=last_hi {
        if dp[last_pi][di] > best_score {
            best_score = dp[last_pi][di];
            last_di = di;
        }
    }

    // Backtrace to collect matched pairs.
    let mut results: Vec<PatchSim> = Vec::with_capacity(num_patches);
    let mut current_di = last_di;

    for pi in (0..num_patches).rev() {
        let ref_start = patch_indices[pi];
        let ref_patch = extract_patch(ref_data, ref_start, patch_size);
        let deg_patch = extract_patch(deg_data, current_di, patch_size);

        let mut sim = nsim::measure(ref_patch.view(), deg_patch.view());
        let prev_di = back[pi][current_di];

        // Detect packet-loss case (no-match when back[pi][di] == di).
        if prev_di == current_di as i64 && pi > 0 {
            sim.similarity = 0.0;
            sim.deg_patch_start_time = 0.0;
            sim.deg_patch_end_time = 0.0;
            sim.freq_band_means.fill(0.0);
        } else {
            sim.deg_patch_start_time = current_di as f64 * frame_duration;
            sim.deg_patch_end_time = sim.deg_patch_start_time + patch_dur;
        }
        sim.ref_patch_start_time = ref_start as f64 * frame_duration;
        sim.ref_patch_end_time = sim.ref_patch_start_time + patch_dur;

        results.push(sim);
        if prev_di >= 0 {
            current_di = prev_di as usize;
        }
    }

    results.reverse();
    Ok(results)
}

/// Drop trailing patches whose start index puts them past the degraded spectrogram end.
/// Matches C++ CalcMaxNumPatches.
fn clamp_patches(
    patch_indices: &[usize],
    n_frames_deg: usize,
    patch_size: usize,
) -> usize {
    let mut n = patch_indices.len();
    while n > 0 {
        let start = patch_indices[n - 1];
        if start.saturating_sub(patch_size / 2) <= n_frames_deg {
            break;
        }
        n -= 1;
    }
    n
}
