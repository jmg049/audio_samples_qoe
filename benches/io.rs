// audio_samples_io read throughput benchmark.
//
// Measures wall-clock time for audio_samples_io::read to decode each
// conformance WAV file from disk as f32.  I/O is included (files are NOT
// pre-loaded); each iteration performs a full open + mmap + parse + decode.
// This is the direct Rust equivalent of the C++ bench_io binary.
//
// Run:
//   cargo bench --bench io

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use audio_samples_io::read;

const TESTDATA: &str = "visqol/testdata/conformance_testdata_subset";

struct File {
    label: &'static str,
    path: &'static str,
}

const FILES: &[File] = &[
    File { label: "strauss_ref",        path: "strauss48_stereo.wav" },
    File { label: "steely_ref",         path: "steely48_stereo.wav" },
    File { label: "contrabassoon_ref",  path: "contrabassoon48_stereo.wav" },
    File { label: "harpsichord_ref",    path: "harpsichord48_stereo.wav" },
    File { label: "guitar_ref",         path: "guitar48_stereo.wav" },
    File { label: "glock_ref",          path: "glock48_stereo.wav" },
    File { label: "ravel_ref",          path: "ravel48_stereo.wav" },
    File { label: "moonlight_ref",      path: "moonlight48_stereo.wav" },
    File { label: "sopr_ref",           path: "sopr48_stereo.wav" },
    File { label: "castanets_ref",      path: "castanets48_stereo.wav" },
    File { label: "steely_lp7",         path: "steely48_stereo_lp7.wav" },
    File { label: "contrabassoon_24aac",path: "contrabassoon48_stereo_24kbps_aac.wav" },
    File { label: "harpsichord_96mp3",  path: "harpsichord48_stereo_96kbps_mp3.wav" },
];

fn bench_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_read_f32");
    group.sample_size(20);

    for file in FILES {
        let path = format!("{TESTDATA}/{}", file.path);
        group.bench_with_input(
            BenchmarkId::from_parameter(file.label),
            &path,
            |b, p| {
                b.iter(|| {
                    let audio = read::<_, f32>(p).unwrap();
                    // Return total_samples so the read is not optimised away.
                    std::hint::black_box(audio.total_samples())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_io);
criterion_main!(benches);
