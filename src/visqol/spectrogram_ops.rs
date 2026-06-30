use ndarray::Array2;

/// Per-frame noise floor: for each frame col, floor = max(ref_peak, deg_peak) - threshold.
pub fn raise_floor_per_frame(
    ref_data: &mut Array2<f64>,
    deg_data: &mut Array2<f64>,
    noise_threshold: f64,
) {
    let min_cols = ref_data.ncols().min(deg_data.ncols());
    for col in 0..min_cols {
        let ref_max = ref_data.column(col).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let deg_max = deg_data.column(col).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let floor = ref_max.max(deg_max) - noise_threshold;
        ref_data.column_mut(col).mapv_inplace(|x| x.max(floor));
        deg_data.column_mut(col).mapv_inplace(|x| x.max(floor));
    }
}

/// Full prepare-for-comparison pipeline.
///
/// Assumes the spectrogram was already returned in dB with a -45 dB floor
/// (via `GammatoneParams::with_db_floor(-45.0)`).  Applies the per-frame
/// dynamic floor then subtracts the global minimum.
pub fn prepare_for_comparison(ref_data: &mut Array2<f64>, deg_data: &mut Array2<f64>) {
    raise_floor_per_frame(ref_data, deg_data, 45.0);

    let global_min = ref_data
        .iter()
        .chain(deg_data.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    ref_data.mapv_inplace(|x| x - global_min);
    deg_data.mapv_inplace(|x| x - global_min);
}
