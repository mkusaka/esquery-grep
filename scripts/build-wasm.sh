#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_wasm="${repo_root}/target/wasm32-wasip1/release/eg.wasm"
output_wasm="${repo_root}/npm/eg.wasm"

if ! command -v wasm-opt >/dev/null 2>&1; then
  echo "wasm-opt is required to build the npm package artifact" >&2
  exit 1
fi

cd "${repo_root}"
cargo build --target wasm32-wasip1 -p esquery-grep --release
wasm-opt -Oz "${target_wasm}" -o "${output_wasm}"

ls -lh "${target_wasm}" "${output_wasm}"
