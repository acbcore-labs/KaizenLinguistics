#!/usr/bin/env bash
set -euo pipefail

export PATH="/opt/homebrew/bin:/Users/kaizenpro/.cargo/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=================================================================="
echo " KaizenLinguistics v2.0 — Full End-to-End Verification Suite"
echo "=================================================================="

echo "[1/6] Testing Deterministic G2P Transducers (grc & hbo)..."
python3 "$ROOT_DIR/compiler/g2p.py"

echo "[2/6] Compiling & Verifying Expanded 140-Concept Course Packs (.klpack + user_telemetry.db)..."
python3 "$ROOT_DIR/compiler/expand_tier23_packs.py"

echo "[3/6] Running Milestone 3 Acoustic Calibration Benchmark (FRR / FAR / Abstention)..."
python3 "$ROOT_DIR/compiler/acoustic_calibration.py"

echo "[4/6] Ingesting & Verifying Open Biblical Corpora & Syntactic Trees (Pillar 3)..."
python3 "$ROOT_DIR/compiler/ingest_corpus.py" --stats --export-pericopes "$ROOT_DIR/data/AUTHENTIC_PERICOPES_CORPUS.json"

echo "[5/6] Running Rust Core & Native Tauri IPC Unit Tests (FSRS-6, Audio, G2P, IPC)..."
cargo test --manifest-path "$ROOT_DIR/app/src-tauri/Cargo.toml"

echo "[6/6] Running Rust Dual-Database Integrity & Runtime Gate..."
cargo run --quiet --manifest-path "$ROOT_DIR/app/src-tauri/Cargo.toml" -- --cli

echo "=================================================================="
echo " ALL 6 VERIFICATION GATES PASSED (ZERO WARNINGS / ZERO ERRORS)"
echo "=================================================================="
