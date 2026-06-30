// Neurogram Similarity Index Measure (NSIM).
// Adapted SSIM metric with 3×3 Gaussian window.
// Ported from ViSQOL's NeurogramSimiliarityIndexMeasure.

use ndarray::{Array1, Array2, ArrayView2, s};

// 3×3 Gaussian kernel matching C++ NSIM weights.
const W: [f64; 9] = [
    0.0113033910173052, 0.0838251475442633, 0.0113033910173052,
    0.0838251475442633, 0.619485845753726,  0.0838251475442633,
    0.0113033910173052, 0.0838251475442633, 0.0113033910173052,
];

const K1: f64 = 0.01;
const K2: f64 = 0.03;
const INTENSITY_RANGE: f64 = 1.0;
const C1: f64 = K1 * K1 * INTENSITY_RANGE * INTENSITY_RANGE;
const C3: f64 = K2 * K2 * INTENSITY_RANGE * INTENSITY_RANGE / 2.0;

/// Result of comparing one patch pair.
pub struct PatchSim {
    /// Mean NSIM across frequency bands (the "similarity" score for this patch).
    pub similarity: f64,
    /// Per-band mean NSIM values.
    pub freq_band_means: Array1<f64>,
    /// Per-band std-dev of NSIM.
    pub freq_band_stddevs: Array1<f64>,
    /// Per-band mean degraded energy.
    pub freq_band_deg_energy: Array1<f64>,
    pub ref_patch_start_time: f64,
    pub ref_patch_end_time: f64,
    pub deg_patch_start_time: f64,
    pub deg_patch_end_time: f64,
}

/// Apply 3×3 Gaussian convolution with boundary extension ("valid with boundary").
/// Each output pixel at (r,c) is the weighted sum of the 3×3 neighbourhood.
/// Edge pixels clamp to the nearest in-bounds value.
fn conv2d_boundary(kernel: &[f64; 9], patch: ArrayView2<f64>) -> Array2<f64> {
    let rows = patch.nrows();
    let cols = patch.ncols();
    let mut out = Array2::<f64>::zeros((rows, cols));
    for r in 0..rows {
        for c in 0..cols {
            let mut sum = 0.0;
            for dr in 0..3usize {
                for dc in 0..3usize {
                    let sr = (r + dr).saturating_sub(1).min(rows - 1);
                    let sc = (c + dc).saturating_sub(1).min(cols - 1);
                    sum += kernel[dr * 3 + dc] * patch[(sr, sc)];
                }
            }
            out[(r, c)] = sum;
        }
    }
    out
}

/// Compute NSIM between a reference and degraded patch.
/// Both patches must have the same shape: `[n_bands, n_frames]`.
pub fn measure(ref_patch: ArrayView2<f64>, deg_patch: ArrayView2<f64>) -> PatchSim {
    let mu_r = conv2d_boundary(&W, ref_patch);
    let mu_d = conv2d_boundary(&W, deg_patch);

    let mu_r_sq = &mu_r * &mu_r;
    let mu_d_sq = &mu_d * &mu_d;
    let mu_rd = &mu_r * &mu_d;

    let ref_sq: Array2<f64> = ref_patch.mapv(|x| x * x);
    let deg_sq: Array2<f64> = deg_patch.mapv(|x| x * x);
    let ref_deg: Array2<f64> = &ref_patch.to_owned() * &deg_patch.to_owned();

    let sigma_r_sq = conv2d_boundary(&W, ref_sq.view()) - &mu_r_sq;
    let sigma_d_sq = conv2d_boundary(&W, deg_sq.view()) - &mu_d_sq;
    let sigma_rd = conv2d_boundary(&W, ref_deg.view()) - &mu_rd;

    // Intensity component.
    let intensity = (2.0 * &mu_rd + C1) / (&mu_r_sq + &mu_d_sq + C1);

    // Structure component.
    let struct_numer = &sigma_rd + C3;
    let struct_denom: Array2<f64> = (&sigma_r_sq * &sigma_d_sq).mapv(|d| {
        if d < 0.0 { C3 } else { d.sqrt() + C3 }
    });
    let structure = struct_numer / struct_denom;

    let sim_map = intensity * structure;

    // Per-band statistics (mean over frame dimension = axis 1).
    let n_bands = ref_patch.nrows();
    let n_frames = ref_patch.ncols();

    let freq_band_means: Array1<f64> = (0..n_bands)
        .map(|b| sim_map.slice(s![b, ..]).mean().unwrap_or(0.0))
        .collect();

    let freq_band_stddevs: Array1<f64> = (0..n_bands)
        .map(|b| {
            let row = sim_map.slice(s![b, ..]);
            let m = freq_band_means[b];
            let var = row.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / n_frames as f64;
            var.sqrt()
        })
        .collect();

    let freq_band_deg_energy: Array1<f64> = (0..n_bands)
        .map(|b| deg_patch.slice(s![b, ..]).mean().unwrap_or(0.0))
        .collect();

    let similarity = freq_band_means.mean().unwrap_or(0.0);

    PatchSim {
        similarity,
        freq_band_means,
        freq_band_stddevs,
        freq_band_deg_energy,
        ref_patch_start_time: 0.0,
        ref_patch_end_time: 0.0,
        deg_patch_start_time: 0.0,
        deg_patch_end_time: 0.0,
    }
}
