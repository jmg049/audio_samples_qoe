// ViSQOL end-to-end benchmarks.
//
// These cover the same file pairs as tests/conformance.rs so timing results
// are directly comparable to the C++ reference implementation.
//
// Run:
//   cargo bench --bench visqol
//
// Compare with C++ (once built via `bazel build //src:visqol`):
//   ./benches/compare_cpp.sh

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

use audio_samples::AudioSamples;
use audio_samples_io::read;
use audio_samples_qoe::{AudioSamplesQoE, VisqolOptions};

const CONFORMANCE: &str = "visqol/testdata/conformance_testdata_subset";
const TESTDATA: &str = "visqol/testdata";

struct Pair {
    label: &'static str,
    reference: &'static str,
    degraded: &'static str,
}

/// All audio-mode conformance pairs, ordered low → high quality so the report
/// spans the full MOS range (1.4 … 4.7).
const PAIRS: &[Pair] = &[
    Pair {
        label: "strauss_lp35",
        reference: "conformance_testdata_subset/strauss48_stereo.wav",
        degraded: "conformance_testdata_subset/strauss48_stereo_lp35.wav",
    },
    Pair {
        label: "steely_lp7",
        reference: "conformance_testdata_subset/steely48_stereo.wav",
        degraded: "conformance_testdata_subset/steely48_stereo_lp7.wav",
    },
    Pair {
        label: "contrabassoon_24aac",
        reference: "conformance_testdata_subset/contrabassoon48_stereo.wav",
        degraded: "conformance_testdata_subset/contrabassoon48_stereo_24kbps_aac.wav",
    },
    Pair {
        label: "harpsichord_96mp3",
        reference: "conformance_testdata_subset/harpsichord48_stereo.wav",
        degraded: "conformance_testdata_subset/harpsichord48_stereo_96kbps_mp3.wav",
    },
    Pair {
        label: "guitar_64aac",
        reference: "conformance_testdata_subset/guitar48_stereo.wav",
        degraded: "conformance_testdata_subset/guitar48_stereo_64kbps_aac.wav",
    },
    Pair {
        label: "glock_48aac",
        reference: "conformance_testdata_subset/glock48_stereo.wav",
        degraded: "conformance_testdata_subset/glock48_stereo_48kbps_aac.wav",
    },
    Pair {
        label: "ravel_128opus",
        reference: "conformance_testdata_subset/ravel48_stereo.wav",
        degraded: "conformance_testdata_subset/ravel48_stereo_128kbps_opus.wav",
    },
    Pair {
        label: "moonlight_128aac",
        reference: "conformance_testdata_subset/moonlight48_stereo.wav",
        degraded: "conformance_testdata_subset/moonlight48_stereo_128kbps_aac.wav",
    },
    Pair {
        label: "sopr_256aac",
        reference: "conformance_testdata_subset/sopr48_stereo.wav",
        degraded: "conformance_testdata_subset/sopr48_stereo_256kbps_aac.wav",
    },
    Pair {
        label: "castanets_identity",
        reference: "conformance_testdata_subset/castanets48_stereo.wav",
        degraded: "conformance_testdata_subset/castanets48_stereo.wav",
    },
    Pair {
        label: "guitar_short_deg",
        reference: "conformance_testdata_subset/guitar48_stereo.wav",
        degraded: "short_duration/5_second/guitar48_stereo_5_sec.wav",
    },
];

fn load(rel: &str) -> AudioSamples<'static, f32> {
    let path = format!("{TESTDATA}/{rel}");
    read::<_, f32>(&path).unwrap_or_else(|e| panic!("load {path}: {e}"))
}

/// End-to-end benchmark: one `visqol()` call per iteration.
/// Files are loaded once and cloned for each iteration so I/O is excluded.
fn bench_end_to_end(c: &mut Criterion) {
    let opts = VisqolOptions::audio();
    let mut group = c.benchmark_group("visqol_audio");

    // Audio processing is slow; 10 samples is enough for a stable estimate.
    group.sample_size(10);

    for pair in PAIRS {
        let ref_audio = load(pair.reference);
        let deg_audio = load(pair.degraded);

        group.bench_with_input(
            BenchmarkId::from_parameter(pair.label),
            pair.label,
            |b, _| {
                b.iter_batched(
                    // Setup: clone pre-loaded audio so disk I/O is not measured.
                    || deg_audio.clone(),
                    // Routine: full ViSQOL pipeline.
                    |deg| ref_audio.visqol_with_options(&deg, &opts).unwrap(),
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_end_to_end);
criterion_main!(benches);
