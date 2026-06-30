/// Evaluate ViSQOL (speech mode) on the genspeech and tcdvoip datasets.
///
/// Usage:
///   cargo run --example eval_datasets --release -- <datasets_root>
///
/// Example:
///   cargo run --example eval_datasets --release -- /home/jmg/code/audio/datasets
///
/// Reports per-sample predicted vs ground-truth MOS, then Pearson r, RMSE, MAE
/// for each dataset.

use audio_samples_io::read;
use audio_samples_qoe::{AudioSamplesQoE, VisqolOptions};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: eval_datasets <datasets_root> [quickstart|full]");
        std::process::exit(1);
    }
    let root = PathBuf::from(&args[1]);
    let mode = args.get(2).map(|s| s.as_str()).unwrap_or("quickstart");

    match mode {
        "quickstart" => {
            println!("=== Genspeech (quickstart, 10 samples) ===");
            run_dataset(
                &root.join("genspeech/quickstart_genspeech.csv"),
                &root.join("genspeech"),
                GtColumn::Named("MOS"),
            );

            println!("\n=== TCDvoip (quickstart, 10 samples) ===");
            run_dataset(
                &root.join("tcdvoip/quickstart_tcdvoip.csv"),
                &root.join("tcdvoip"),
                GtColumn::Named("sampleMOS"),
            );
        }
        "full" => {
            println!("=== Genspeech (full, 160 samples) ===");
            run_dataset(
                &root.join("genspeech/genspeech.csv"),
                &root.join("genspeech"),
                GtColumn::Named("MOS"),
            );

            println!("\n=== TCDvoip (full, 384 samples) ===");
            run_dataset(
                &root.join("tcdvoip/tcdvoip.csv"),
                &root.join("tcdvoip"),
                GtColumn::Named("sampleMOS"),
            );
        }
        other => {
            eprintln!("unknown mode '{other}': use 'quickstart' or 'full'");
            std::process::exit(1);
        }
    }
}

enum GtColumn {
    Named(&'static str),
}

struct Row {
    ref_path: PathBuf,
    deg_path: PathBuf,
    mos: f64,
}

fn parse_csv(csv_path: &Path, base: &Path, gt_col: &GtColumn) -> Vec<Row> {
    let f = File::open(csv_path).unwrap_or_else(|e| {
        eprintln!("cannot open {}: {e}", csv_path.display());
        std::process::exit(1);
    });
    let mut lines = BufReader::new(f).lines();

    let header_line = lines.next().unwrap().unwrap();
    let headers: Vec<&str> = header_line.split(',').collect();

    let ref_idx = headers.iter().position(|h| h.trim() == "Ref_Wave").unwrap();
    let deg_idx = headers.iter().position(|h| h.trim() == "Test_Wave").unwrap();
    let mos_idx = match gt_col {
        GtColumn::Named(name) => headers.iter().position(|h| h.trim() == *name).unwrap(),
    };

    let mut rows = Vec::new();
    for line in lines {
        let line = line.unwrap();
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() <= mos_idx {
            continue;
        }
        let ref_rel = cols[ref_idx].trim();
        let deg_rel = cols[deg_idx].trim();
        let mos_str = cols[mos_idx].trim();
        if ref_rel.is_empty() || deg_rel.is_empty() || mos_str.is_empty() {
            continue;
        }
        let mos: f64 = match mos_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        rows.push(Row {
            ref_path: base.join(ref_rel),
            deg_path: base.join(deg_rel),
            mos,
        });
    }
    rows
}

fn run_dataset(csv_path: &Path, base: &Path, gt_col: GtColumn) {
    let rows = parse_csv(csv_path, base, &gt_col);
    let opts = VisqolOptions::speech();

    let mut predicted: Vec<f64> = Vec::new();
    let mut ground_truth: Vec<f64> = Vec::new();

    for row in &rows {
        let ref_audio = match read::<_, f32>(&row.ref_path) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("  SKIP {}: {e}", row.ref_path.display());
                continue;
            }
        };
        let deg_audio = match read::<_, f32>(&row.deg_path) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("  SKIP {}: {e}", row.deg_path.display());
                continue;
            }
        };

        match ref_audio.visqol_with_options(&deg_audio, &opts) {
            Ok(mos_hat) => {
                println!(
                    "  {ref} | GT {gt:.3} | pred {pred:.3} | Δ {delta:+.3}",
                    ref = row.ref_path.file_name().unwrap().to_string_lossy(),
                    gt = row.mos,
                    pred = mos_hat,
                    delta = mos_hat - row.mos,
                );
                predicted.push(mos_hat);
                ground_truth.push(row.mos);
            }
            Err(e) => {
                eprintln!(
                    "  ERROR {} vs {}: {e}",
                    row.ref_path.display(),
                    row.deg_path.display()
                );
            }
        }
    }

    if predicted.is_empty() {
        println!("  No results.");
        return;
    }

    let n = predicted.len() as f64;
    let mae = predicted.iter().zip(&ground_truth).map(|(p, g)| (p - g).abs()).sum::<f64>() / n;
    let rmse = (predicted.iter().zip(&ground_truth).map(|(p, g)| (p - g).powi(2)).sum::<f64>() / n).sqrt();
    let r = pearson(&predicted, &ground_truth);

    println!();
    println!("  Samples : {}", predicted.len());
    println!("  Pearson r: {r:.4}");
    println!("  RMSE     : {rmse:.4}");
    println!("  MAE      : {mae:.4}");
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let num = x.iter().zip(y).map(|(xi, yi)| (xi - mx) * (yi - my)).sum::<f64>();
    let sx = x.iter().map(|xi| (xi - mx).powi(2)).sum::<f64>().sqrt();
    let sy = y.iter().map(|yi| (yi - my).powi(2)).sum::<f64>().sqrt();
    if sx == 0.0 || sy == 0.0 { return 0.0; }
    num / (sx * sy)
}
