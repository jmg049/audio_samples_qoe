// CLI wrapper for SCOREQ, mirroring upstream `python -m scoreq`.
// Usage:
//   scoreq_cli <natural|synthetic> <nr|ref> <test.wav> [reference.wav]
//
// NR mode prints a predicted MOS; REF mode prints the embedding distance to the
// reference (requires the reference argument). First use of a model downloads
// ~378 MB of weights into ~/.cache/scoreq/.
//
// Build with: cargo run --example scoreq_cli --features scoreq -- ...

#[cfg(not(feature = "scoreq"))]
fn main() {
    eprintln!("rebuild with --features scoreq");
    std::process::exit(1);
}

#[cfg(feature = "scoreq")]
fn main() {
    use audio_samples_io::read;
    use audio_samples_qoe::{Scoreq, ScoreqDomain, ScoreqMode};

    let args: Vec<String> = std::env::args().collect();
    if !(4..=5).contains(&args.len()) {
        eprintln!("usage: scoreq_cli <natural|synthetic> <nr|ref> <test.wav> [reference.wav]");
        std::process::exit(1);
    }

    let domain = match args[1].as_str() {
        "natural" => ScoreqDomain::Natural,
        "synthetic" => ScoreqDomain::Synthetic,
        other => fail(&format!("unknown domain '{other}' (expected natural|synthetic)")),
    };
    let mode = match args[2].as_str() {
        "nr" => ScoreqMode::Nr,
        "ref" => ScoreqMode::Ref,
        other => fail(&format!("unknown mode '{other}' (expected nr|ref)")),
    };

    let test = read::<_, f32>(&args[3])
        .unwrap_or_else(|e| fail(&format!("failed to load {}: {e}", args[3])));

    let mut model = Scoreq::new(domain, mode)
        .unwrap_or_else(|e| fail(&format!("failed to load model: {e}")));

    let score = match mode {
        ScoreqMode::Nr => model
            .predict(&test)
            .unwrap_or_else(|e| fail(&format!("predict error: {e}"))),
        ScoreqMode::Ref => {
            let ref_path = args.get(4).unwrap_or_else(|| fail("ref mode requires a reference.wav argument"));
            let reference = read::<_, f32>(ref_path)
                .unwrap_or_else(|e| fail(&format!("failed to load {ref_path}: {e}")));
            model
                .predict_ref(&test, &reference)
                .unwrap_or_else(|e| fail(&format!("predict_ref error: {e}")))
        }
    };

    println!("{score:.10}");
}

#[cfg(feature = "scoreq")]
fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}
