#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_wasm="${repo_root}/target/wasm32-wasip1/release/eg.wasm"
output_wasm="${repo_root}/npm/eg.wasm"
cargo_version="$(
  sed -n 's/^version = "\(.*\)"/\1/p' "${repo_root}/crates/esquery-grep/Cargo.toml" | head -n 1
)"

if ! command -v wasm-opt >/dev/null 2>&1; then
  echo "wasm-opt is required to build the npm package artifact" >&2
  exit 1
fi

cd "${repo_root}"
export EG_VERSION="${EG_VERSION:-${cargo_version}}"
cargo build --target wasm32-wasip1 -p esquery-grep --release
wasm-opt -Oz "${target_wasm}" -o "${output_wasm}"

echo "Built npm/eg.wasm with EG_VERSION=${EG_VERSION}"
ls -lh "${target_wasm}" "${output_wasm}"
