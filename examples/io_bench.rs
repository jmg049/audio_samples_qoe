// Standalone I/O benchmark: reads each WAV file given on argv using
// audio_samples_io::read, then prints the total sample count to stdout.
//
// Usage:  io_bench <file1.wav> [file2.wav ...]
// Timing: wrap with hyperfine for per-invocation wall-clock measurement.

use audio_samples_io::read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: io_bench <file.wav> ...");
        std::process::exit(1);
    }

    let mut total: usize = 0;
    for path in &args {
        let audio = read::<_, f32>(path)
            .unwrap_or_else(|e| { eprintln!("failed to load {path}: {e}"); std::process::exit(1); });
        total += audio.total_samples().get();
    }

    println!("{total}");
}
