#!/usr/bin/env bash
# Compare Rust ViSQOL throughput and I/O against the C++ reference (SVR mode).
#
# Produces three measurements per conformance pair:
#   1. End-to-end wall-clock  — full process: startup + I/O + pipeline
#   2. I/O-only wall-clock    — just WAV decode (no pipeline)
#   3. Criterion pure-compute — pipeline only, I/O excluded (cargo bench)
#
# Optimization parity:
#   C++  : -O3 -march=native  (Makefile.svr)
#   Rust : opt-level=3, codegen-units=1, lto=thin, target-cpu=native
#          (.cargo/config.toml + Cargo.toml [profile.release])
#
# Prerequisites:
#   C++ pipeline binary:  cd visqol && make -f Makefile.svr
#   C++ I/O benchmark:    cd visqol && make -f Makefile.svr bench_io
#   hyperfine:            cargo install hyperfine  OR  pacman -S hyperfine
#
# Usage:
#   ./benches/compare_cpp.sh                    # console output only
#   ./benches/compare_cpp.sh --save             # also write results/YYYY-MM-DD.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TESTDATA="${REPO_ROOT}/visqol/testdata"
CONFORMANCE="${TESTDATA}/conformance_testdata_subset"

CPP_BIN="${REPO_ROOT}/visqol/build/visqol_svr"
CPP_IO_BIN="${REPO_ROOT}/visqol/build/bench_io"
CPP_MODEL="${REPO_ROOT}/visqol/model/libsvm_nu_svr_model.txt"

RUST_BIN="${REPO_ROOT}/target/release/examples/visqol_cli"
RUST_IO_BIN="${REPO_ROOT}/target/release/examples/io_bench"

SAVE=0
EXTRA_ARGS=()
for arg in "$@"; do
    if [[ "$arg" == "--save" ]]; then
        SAVE=1
    else
        EXTRA_ARGS+=("$arg")
    fi
done

# ── Sanity checks ─────────────────────────────────────────────────────────────

for bin in "$CPP_BIN" "$CPP_IO_BIN"; do
    if [[ ! -x "$bin" ]]; then
        echo "Binary not found: $bin"
        echo "Build with:  cd visqol && make -f Makefile.svr all bench_io"
        exit 1
    fi
done

if ! command -v hyperfine &>/dev/null; then
    echo "hyperfine not found — install with: pacman -S hyperfine  or  cargo install hyperfine"
    exit 1
fi

# ── Build release Rust binaries ───────────────────────────────────────────────

echo "Building Rust release binaries..."
cargo build --release \
    --example visqol_cli \
    --example io_bench \
    --manifest-path "${REPO_ROOT}/Cargo.toml" 2>&1

# ── System info ───────────────────────────────────────────────────────────────

CPU_MODEL="$(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs)"
KERNEL="$(uname -r)"
RUSTC_VER="$(rustc --version)"
GCC_VER="$(g++ --version | head -1)"
DATE="$(date -u +%Y-%m-%d)"
HOSTNAME_SHORT="$(hostname -s)"

RUST_FLAGS="opt-level=3, codegen-units=1, lto=thin, target-cpu=native"
CPP_FLAGS="-O3 -march=native -std=c++17"

print_sysinfo() {
    echo ""
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                     System Info                         ║"
    echo "╠══════════════════════════════════════════════════════════╣"
    printf "║  CPU    : %-47s ║\n" "$CPU_MODEL"
    printf "║  Kernel : %-47s ║\n" "$KERNEL"
    printf "║  rustc  : %-47s ║\n" "$RUSTC_VER"
    printf "║  g++    : %-47s ║\n" "${GCC_VER:0:47}"
    printf "║  Rust   : %-47s ║\n" "$RUST_FLAGS"
    printf "║  C++    : %-47s ║\n" "$CPP_FLAGS"
    printf "║  Date   : %-47s ║\n" "$DATE"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    echo "Normalization note:"
    echo "  C++  : i16 / 32768.0  (symmetric)"
    echo "  Rust : i16 / 32768.0  (negative) | i16 / 32767.0 (positive)"
    echo "  Amplitude difference: ~0.003% — negligible for MOS"
    echo ""
}

print_sysinfo

# ── File pairs ────────────────────────────────────────────────────────────────

declare -A PAIRS=(
    ["strauss_lp35"]="${CONFORMANCE}/strauss48_stereo.wav ${CONFORMANCE}/strauss48_stereo_lp35.wav"
    ["steely_lp7"]="${CONFORMANCE}/steely48_stereo.wav ${CONFORMANCE}/steely48_stereo_lp7.wav"
    ["contrabassoon_24aac"]="${CONFORMANCE}/contrabassoon48_stereo.wav ${CONFORMANCE}/contrabassoon48_stereo_24kbps_aac.wav"
    ["harpsichord_96mp3"]="${CONFORMANCE}/harpsichord48_stereo.wav ${CONFORMANCE}/harpsichord48_stereo_96kbps_mp3.wav"
    ["guitar_64aac"]="${CONFORMANCE}/guitar48_stereo.wav ${CONFORMANCE}/guitar48_stereo_64kbps_aac.wav"
    ["glock_48aac"]="${CONFORMANCE}/glock48_stereo.wav ${CONFORMANCE}/glock48_stereo_48kbps_aac.wav"
    ["ravel_128opus"]="${CONFORMANCE}/ravel48_stereo.wav ${CONFORMANCE}/ravel48_stereo_128kbps_opus.wav"
    ["moonlight_128aac"]="${CONFORMANCE}/moonlight48_stereo.wav ${CONFORMANCE}/moonlight48_stereo_128kbps_aac.wav"
    ["sopr_256aac"]="${CONFORMANCE}/sopr48_stereo.wav ${CONFORMANCE}/sopr48_stereo_256kbps_aac.wav"
    ["castanets_identity"]="${CONFORMANCE}/castanets48_stereo.wav ${CONFORMANCE}/castanets48_stereo.wav"
    ["guitar_short_deg"]="${CONFORMANCE}/guitar48_stereo.wav ${TESTDATA}/short_duration/5_second/guitar48_stereo_5_sec.wav"
)

# Unique set of individual files for I/O benchmark (refs + degs deduplicated)
declare -A IO_FILES=()
for pair in "${PAIRS[@]}"; do
    read -r ref_file deg_file <<< "$pair"
    IO_FILES["$ref_file"]=1
    IO_FILES["$deg_file"]=1
done

# ── Section 1: End-to-end (includes I/O + startup) ────────────────────────────

echo "════════════════════════════════════════════════════════════"
echo "  SECTION 1 — End-to-end  (startup + I/O + pipeline)"
echo "  Both processes: read 2 WAV files, compute MOS, print, exit"
echo "════════════════════════════════════════════════════════════"

if [[ $SAVE -eq 1 ]]; then
    mkdir -p "${REPO_ROOT}/results"
    RESULT_FILE="${REPO_ROOT}/results/${DATE}_${HOSTNAME_SHORT}_e2e.md"
    E2E_EXPORT=("--export-markdown" "$RESULT_FILE")
    echo "(saving to $RESULT_FILE)"
else
    E2E_EXPORT=()
fi

for label in "${!PAIRS[@]}"; do
    read -r ref_file deg_file <<< "${PAIRS[$label]}"
    echo ""
    echo "─── ${label} ───"
    hyperfine \
        --warmup 1 \
        --min-runs 10 \
        --command-name "rust:${label}" "${RUST_BIN} ${ref_file} ${deg_file}" \
        --command-name "cpp:${label}"  "${CPP_BIN} --reference_file ${ref_file} --degraded_file ${deg_file} --use_lattice_model=false --similarity_to_quality_model=${CPP_MODEL}" \
        "${E2E_EXPORT[@]+"${E2E_EXPORT[@]}"}" \
        "${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"}"
done

# ── Section 2: I/O-only ───────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  SECTION 2 — I/O only  (open + mmap/stream + decode + normalize)"
echo "  Rust: audio_samples_io  |  C++: WavReader + NormalizeInt16ToDouble"
echo "  Each invocation reads ALL conformance files in one pass."
echo "════════════════════════════════════════════════════════════"

ALL_FILES="${!IO_FILES[@]}"

if [[ $SAVE -eq 1 ]]; then
    IO_RESULT_FILE="${REPO_ROOT}/results/${DATE}_${HOSTNAME_SHORT}_io.md"
    IO_EXPORT=("--export-markdown" "$IO_RESULT_FILE")
    echo "(saving to $IO_RESULT_FILE)"
else
    IO_EXPORT=()
fi

echo ""
echo "─── all_conformance_files ───"
hyperfine \
    --warmup 2 \
    --min-runs 15 \
    --command-name "rust:io" "${RUST_IO_BIN} ${ALL_FILES}" \
    --command-name "cpp:io"  "${CPP_IO_BIN} ${ALL_FILES}" \
    "${IO_EXPORT[@]+"${IO_EXPORT[@]}"}" \
    "${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"}"

# ── Section 3: Pure compute (Criterion) ───────────────────────────────────────

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  SECTION 3 — Pure pipeline compute  (no I/O, no startup)"
echo "  Rust only: Criterion benchmark, files pre-loaded, I/O excluded."
echo "  C++ equivalent: not implemented (would require a C++ harness)."
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Running cargo bench --bench visqol ..."
echo "(results in target/criterion/visqol_audio/)"
echo ""

if [[ $SAVE -eq 1 ]]; then
    CRITERION_OUT="${REPO_ROOT}/results/${DATE}_${HOSTNAME_SHORT}_criterion.txt"
    cargo bench --bench visqol --manifest-path "${REPO_ROOT}/Cargo.toml" 2>&1 | tee "$CRITERION_OUT"
    echo ""
    echo "(saved to $CRITERION_OUT)"
else
    cargo bench --bench visqol --manifest-path "${REPO_ROOT}/Cargo.toml"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  SUMMARY"
echo "════════════════════════════════════════════════════════════"
echo "  CPU      : $CPU_MODEL"
echo "  Rust     : $RUST_FLAGS"
echo "  C++      : $CPP_FLAGS"
echo "  rustc    : $RUSTC_VER"
echo "  g++      : $GCC_VER"
echo ""
echo "  Sections:"
echo "    1. End-to-end   — hyperfine, 10 runs, 1 warmup (per pair)"
echo "    2. I/O-only     — hyperfine, 15 runs, 2 warmup (all files, one pass)"
echo "    3. Pure compute — Criterion (Rust only; C++ harness not implemented)"
echo ""
echo "  Interpretation:"
echo "    Section 1 - Section 2 ≈ pipeline compute time (approximate)"
echo "    Section 3 gives the precise Rust pipeline time without I/O noise"
if [[ $SAVE -eq 1 ]]; then
    echo ""
    echo "  Results saved:"
    echo "    $RESULT_FILE"
    echo "    $IO_RESULT_FILE"
    echo "    $CRITERION_OUT"
fi
echo "════════════════════════════════════════════════════════════"
