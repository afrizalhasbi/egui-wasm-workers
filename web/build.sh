#!/bin/sh
# Build the wasm app into web/pkg/, ready to serve with `python web/serve.py`.
# Requires: rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli
set -e

cd "$(dirname "$0")/.."

cargo build --lib --target wasm32-unknown-unknown --release

wasm-bindgen \
  target/wasm32-unknown-unknown/release/egui_wasm_workers.wasm \
  --out-dir web/pkg \
  --target web

# Post-process for size. Requires binaryen: pacman -S binaryen
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --enable-bulk-memory --enable-mutable-globals --strip-debug \
    web/pkg/egui_wasm_workers_bg.wasm -o web/pkg/egui_wasm_workers_bg.wasm
else
  echo "wasm-opt not found, skipping size pass (install: pacman -S binaryen)"
fi

echo "done: serve with python web/serve.py, open http://localhost:8123"
