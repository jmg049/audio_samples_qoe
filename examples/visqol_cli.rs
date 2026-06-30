// Minimal CLI wrapper used by benches/compare_cpp.sh.
// Usage: visqol_cli <reference.wav> <degraded.wav>
// Prints the MOS-LQO score to stdout.

use audio_samples_io::read;
use audio_samples_qoe::{AudioSamplesQoE, VisqolOptions};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: visqol_cli <reference.wav> <degraded.wav>");
        std::process::exit(1);
    }

    let ref_audio = read::<_, f32>(&args[1])
        .unwrap_or_else(|e| { eprintln!("failed to load {}: {e}", args[1]); std::process::exit(1); });
    let deg_audio = read::<_, f32>(&args[2])
        .unwrap_or_else(|e| { eprintln!("failed to load {}: {e}", args[2]); std::process::exit(1); });

    let opts = VisqolOptions::audio();
    let start_time = std::time::Instant::now();
    let mos = ref_audio
        .visqol_with_options(&deg_audio, &opts)
        .unwrap_or_else(|e| { eprintln!("visqol error: {e}"); std::process::exit(1); });
    let elapsed = start_time.elapsed();
    eprintln!("visqol took {elapsed:.2?}");
    println!("{mos:.10}");
}
