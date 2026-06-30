"""Score two audio files and print the MOS-LQO.

Usage:
    python examples/01_basic_file_scoring.py reference.wav degraded.wav
    python examples/01_basic_file_scoring.py reference.wav degraded.wav --mode speech
"""

import argparse
import sys
from pathlib import Path

from audio_samples_qoe import VisqolError, visqol


def main() -> None:
    parser = argparse.ArgumentParser(description="ViSQOL file scorer")
    parser.add_argument("reference", type=Path, help="Clean reference file (WAV/FLAC)")
    parser.add_argument("degraded", type=Path, help="Degraded file to evaluate (WAV/FLAC)")
    parser.add_argument(
        "--mode",
        choices=["audio", "speech"],
        default="audio",
        help="Scoring mode (default: audio)",
    )
    args = parser.parse_args()

    try:
        score = visqol(args.reference, args.degraded, mode=args.mode)
    except FileNotFoundError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    except VisqolError as e:
        print(f"scoring failed: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"MOS-LQO ({args.mode}): {score:.6f}")


if __name__ == "__main__":
    main()
