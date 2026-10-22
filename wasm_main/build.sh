#!/usr/bin/env bash
# Builds the wasm bundle into wasm_main/dist (index.html + pkg/). Requires wasm-bindgen-cli matching Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p wasm_main --target wasm32-unknown-unknown --release
rm -rf wasm_main/dist
mkdir -p wasm_main/dist
wasm-bindgen --target web --out-dir wasm_main/dist/pkg target/wasm32-unknown-unknown/release/wasm_main.wasm
cp wasm_main/web/index.html wasm_main/dist/
