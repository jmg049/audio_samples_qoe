// Speech-mode reference patch selection, gated on voice activity.
// Ported from ViSQOL's RmsVad + VadPatchCreator (rms_vad.cc, vad_patch_creator.cc).

/// Consecutive sub-threshold chunks required before a chunk is marked silent.
const SILENT_CHUNK_COUNT: usize = 3;
/// RMS threshold in int16 sample units (the C++ VAD operates on PCM16 values).
const RMS_THRESHOLD: f64 = 5000.0;
/// Frames with voice activity required for a patch to be included.
const FRAMES_WITH_VA_THRESHOLD: f64 = 1.0;

/// Reference patch indices for speech mode: the standard patch grid
/// (`first = patch_size / 2 - 1`, stride `patch_size`) filtered to patches
/// whose frames contain voice activity.
///
/// `ref_samples` is the prepared (mono, mode-rate) reference signal *before*
/// SPL scaling — the same signal the spectrogram is computed from.
/// `n_frames` is the spectrogram column count and `frame_len` the STFT hop.
pub fn vad_ref_patch_indices(
    ref_samples: &[f64],
    n_frames: usize,
    patch_size: usize,
    frame_len: usize,
) -> Vec<usize> {
    let first_patch_idx = match (patch_size / 2).checked_sub(1) {
        Some(i) => i,
        None => return vec![],
    };
    if n_frames <= first_patch_idx {
        return vec![];
    }
    let patch_count = (n_frames - first_patch_idx) / patch_size;
    if patch_count == 0 {
        return vec![];
    }

    // C++ normalises by the maximum element (not the maximum magnitude).
    let max = ref_samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // C++ starts the VAD at sample index `first_patch_idx` (not frame index —
    // replicated verbatim for conformance) and feeds whole frames only.
    let total_samples = patch_count * patch_size * frame_len;
    let start = first_patch_idx.min(ref_samples.len());
    let end = (start + total_samples).min(ref_samples.len());

    let vad = rms_vad(ref_samples[start..end].chunks_exact(frame_len).map(|frame| {
        frame.iter().map(|&v| {
            // Scale to PCM16 range, clamp, truncate — matching the C++
            // double → int16 narrowing conversion.
            let v = (v / max) * f64::from(1u32 << 15);
            v.clamp(-32768.0, 32767.0).trunc()
        })
    }));

    (0..patch_count)
        .filter(|&i| {
            let frames_with_va: f64 = vad
                .iter()
                .skip(i * patch_size)
                .take(patch_size)
                .sum();
            frames_with_va >= FRAMES_WITH_VA_THRESHOLD
        })
        .map(|i| first_patch_idx + i * patch_size)
        .collect()
}

/// Per-chunk voice activity (1.0 = present, 0.0 = absent).
///
/// A chunk is marked absent only when it and the previous
/// `SILENT_CHUNK_COUNT - 1` chunks are all below the RMS threshold; the first
/// `SILENT_CHUNK_COUNT - 1` chunks are always marked present.
fn rms_vad<I, F>(chunks: I) -> Vec<f64>
where
    I: Iterator<Item = F>,
    F: Iterator<Item = f64>,
{
    let above: Vec<bool> = chunks
        .map(|chunk| {
            let (sum_sq, n) = chunk.fold((0.0, 0usize), |(s, n), v| (s + v * v, n + 1));
            (sum_sq / n as f64).sqrt() >= RMS_THRESHOLD
        })
        .collect();

    let mut results = vec![1.0; (SILENT_CHUNK_COUNT - 1).min(above.len())];
    for i in (SILENT_CHUNK_COUNT - 1)..above.len() {
        let silent = !above[i] && above[i + 1 - SILENT_CHUNK_COUNT..i].iter().all(|&a| !a);
        results.push(if silent { 0.0 } else { 1.0 });
    }
    results
}
