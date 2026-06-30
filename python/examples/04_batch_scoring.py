"""Score a directory of degraded files against a single reference.

Usage:
    python examples/04_batch_scoring.py reference.wav degraded_dir/
    python examples/04_batch_scoring.py reference.wav degraded_dir/ --workers 4
    python examples/04_batch_scoring.py reference.wav degraded_dir/ --csv results.csv

The GIL is released during scoring, so threads saturate all cores.
"""

import argparse
import csv
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from audio_samples_qoe import VisqolError, visqol

SUPPORTED = {".wav", ".flac"}


def score_one(ref: Path, deg: Path, mode: str) -> tuple[str, float | str]:
    try:
        return deg.name, visqol(ref, deg, mode=mode)
    except (OSError, VisqolError) as e:
        return deg.name, f"ERROR: {e}"


def main() -> None:
    parser = argparse.ArgumentParser(description="Batch ViSQOL scorer")
    parser.add_argument("reference", type=Path)
    parser.add_argument("degraded_dir", type=Path)
    parser.add_argument("--mode", choices=["audio", "speech"], default="audio")
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--csv", type=Path, default=None, help="Save results to CSV")
    args = parser.parse_args()

    if not args.reference.is_file():
        print(f"error: reference not found: {args.reference}", file=sys.stderr)
        sys.exit(1)
    if not args.degraded_dir.is_dir():
        print(f"error: not a directory: {args.degraded_dir}", file=sys.stderr)
        sys.exit(1)

    files = sorted(p for p in args.degraded_dir.iterdir() if p.suffix in SUPPORTED)
    if not files:
        print("no WAV/FLAC files found", file=sys.stderr)
        sys.exit(1)

    print(f"Scoring {len(files)} file(s) against {args.reference.name} "
          f"[mode={args.mode}, workers={args.workers}]\n")

    results: list[tuple[str, float | str]] = []

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(score_one, args.reference, f, args.mode): f for f in files}
        for future in as_completed(futures):
            name, result = future.result()
            if isinstance(result, float):
                print(f"  {name:<45} {result:.6f}")
            else:
                print(f"  {name:<45} {result}", file=sys.stderr)
            results.append((name, result))

    results.sort(key=lambda r: r[0])

    if args.csv:
        with open(args.csv, "w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["file", "mos_lqo"])
            writer.writerows(results)
        print(f"\nResults saved to {args.csv}")

    numeric = [r for _, r in results if isinstance(r, float)]
    if numeric:
        print(f"\nMin: {min(numeric):.4f}  Max: {max(numeric):.4f}  "
              f"Mean: {sum(numeric)/len(numeric):.4f}")


if __name__ == "__main__":
    main()
