//! SCOREQ sanity tests.
//!
//! Per the agreed conformance approach, Rust `ort` wraps the same onnxruntime
//! C++ kernels as the upstream Python package, so the model math is canonical by
//! construction. These tests verify the surrounding pipeline: preprocessing
//! shape, score sanity, determinism, and mode invariants.
//!
//! The first run downloads ~378 MB of weights into `~/.cache/scoreq/`.

#![cfg(feature = "scoreq")]

use audio_samples_io::read;
use audio_samples_qoe::{Scoreq, ScoreqDomain, ScoreqMode};

const OPUS: &str = "scoreq/data/opus.wav";
const REF: &str = "scoreq/data/ref.wav";

#[test]
fn natural_nr_opus_is_sane_and_deterministic() {
    let opus = read::<_, f32>(OPUS).expect("load opus.wav");

    let mut model = Scoreq::new(ScoreqDomain::Natural, ScoreqMode::Nr).expect("load NR model");
    let mos1 = model.predict(&opus).expect("predict 1");
    let mos2 = model.predict(&opus).expect("predict 2");

    eprintln!("natural/NR opus.wav MOS = {mos1}");
    assert!(mos1.is_finite(), "MOS must be finite");
    assert!(
        (1.0..=5.0).contains(&mos1),
        "MOS {mos1} outside plausible [1, 5] range"
    );
    assert_eq!(mos1, mos2, "inference must be deterministic");
}

#[test]
fn nr_model_rejects_ref_call() {
    let opus = read::<_, f32>(OPUS).expect("load opus.wav");
    let mut model = Scoreq::new(ScoreqDomain::Natural, ScoreqMode::Nr).expect("load NR model");
    assert!(
        model.predict_ref(&opus, &opus).is_err(),
        "Nr model must reject predict_ref()"
    );
}

#[test]
#[ignore = "downloads a second ~377 MB model"]
fn natural_ref_distance_is_sane() {
    let opus = read::<_, f32>(OPUS).expect("load opus.wav");
    let reference = read::<_, f32>(REF).expect("load ref.wav");

    let mut model = Scoreq::new(ScoreqDomain::Natural, ScoreqMode::Ref).expect("load REF model");

    let self_dist = model.predict_ref(&reference, &reference).expect("self distance");
    let cross_dist = model.predict_ref(&opus, &reference).expect("cross distance");

    eprintln!("REF self={self_dist} cross={cross_dist}");
    assert!(self_dist < 1e-4, "distance to itself should be ~0, got {self_dist}");
    assert!(cross_dist > self_dist, "degraded should be farther than self");
    assert!(model.predict(&opus).is_err(), "Ref model must reject predict()");
}

#[test]
#[ignore = "downloads the ~378 MB synthetic NR model"]
fn synthetic_nr_opus_is_sane() {
    let opus = read::<_, f32>(OPUS).expect("load opus.wav");
    let mut model =
        Scoreq::new(ScoreqDomain::Synthetic, ScoreqMode::Nr).expect("load synthetic NR model");
    let mos = model.predict(&opus).expect("predict");
    eprintln!("synthetic/NR opus.wav MOS = {mos}");
    assert!(mos.is_finite() && (1.0..=5.0).contains(&mos), "MOS {mos} out of range");
}
