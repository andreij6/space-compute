#!/usr/bin/env bash
# Rebuilds crates/aaa the same way icp.yaml's @dfinity/rust recipe does
# (release profile, wasm32-unknown-unknown, ic-wasm shrink, gzip -n) so the
# resulting sha256 can be checked against a platform-approved WasmMeta.module_sha256
# (crates/platform/src/registry.rs). Prefers a pinned container; falls back to a
# hermetic local build (isolated CARGO_HOME/target, remapped absolute paths) when
# Docker/Podman isn't available. See docs/ops/aaa-manual-upgrade.md.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IC_WASM_VERSION="0.11.1"
RUST_VERSION="$(sed -nE 's/^channel = "(.*)"/\1/p' "$ROOT/rust-toolchain.toml")"

require_pinned_versions() {
  local rustc_v icwasm_v
  rustc_v="$(rustc --version | awk '{print $2}')"
  if [ "$rustc_v" != "$RUST_VERSION" ]; then
    echo "rustc $rustc_v does not match the pinned $RUST_VERSION (rustup toolchain install $RUST_VERSION)" >&2
    exit 1
  fi
  icwasm_v="$(ic-wasm --version | awk '{print $2}')"
  if [ "$icwasm_v" != "$IC_WASM_VERSION" ]; then
    echo "ic-wasm $icwasm_v does not match the pinned $IC_WASM_VERSION (cargo install ic-wasm --version $IC_WASM_VERSION --locked)" >&2
    exit 1
  fi
}

build_core() {
  local src="$1" out="$2"
  require_pinned_versions
  mkdir -p "$out"
  local build_root
  build_root="$(mktemp -d "${out%/}/.build.XXXXXX")"
  trap 'rm -rf "$build_root"' RETURN

  local cargo_home="$build_root/cargo-home" target_dir="$build_root/target"
  local seed="${CARGO_HOME:-$HOME/.cargo}"
  mkdir -p "$cargo_home"
  [ -d "$seed/registry" ] && ln -sfn "$seed/registry" "$cargo_home/registry"
  [ -d "$seed/git" ] && ln -sfn "$seed/git" "$cargo_home/git"

  local src_abs sysroot epoch
  src_abs="$(cd "$src" && pwd)"
  sysroot="$(rustc --print sysroot)"
  epoch="$(cd "$src_abs" && git log -1 --format=%ct 2>/dev/null || echo 0)"

  (
    cd "$src_abs"
    export CARGO_HOME="$cargo_home"
    export CARGO_TARGET_DIR="$target_dir"
    export SOURCE_DATE_EPOCH="$epoch"
    export RUSTFLAGS="--remap-path-prefix=${src_abs}=/build/space-compute --remap-path-prefix=${cargo_home}=/build/cargo-home --remap-path-prefix=${sysroot}=/build/rust-sysroot"
    cargo build --locked --release --target wasm32-unknown-unknown -p aaa >&2
  )

  ic-wasm "$target_dir/wasm32-unknown-unknown/release/aaa.wasm" -o "$out/aaa.wasm" shrink >&2
  gzip -n -c "$out/aaa.wasm" >"$out/aaa.wasm.gz"
}

print_hashes() {
  local out="$1"
  echo "module_sha256=$(shasum -a 256 "$out/aaa.wasm" | awk '{print $1}')"
  echo "gz_sha256=$(shasum -a 256 "$out/aaa.wasm.gz" | awk '{print $1}')"
}

if [ "${1:-}" = "--core" ]; then
  build_core "$2" "$3"
  print_hashes "$3"
  exit 0
fi

SRC_DIR="${1:-$ROOT}"
OUT_DIR="${2:-$ROOT/target/aaa-reproducible/local}"
MODE="${AAA_BUILD_MODE:-auto}"

use_container() {
  [ "$MODE" = "local" ] && return 1
  if [ "$MODE" = "docker" ] || [ "$MODE" = "auto" ]; then
    command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1
  else
    return 1
  fi
}

mkdir -p "$OUT_DIR"
SRC_ABS="$(cd "$SRC_DIR" && pwd)"

if use_container; then
  echo "space-compute: building crates/aaa in the pinned container (rust ${RUST_VERSION}, ic-wasm ${IC_WASM_VERSION})" >&2
  docker build -q -f "$ROOT/Dockerfile.aaa-reproducible" --build-arg IC_WASM_VERSION="$IC_WASM_VERSION" -t sc-aaa-reproducible-build "$ROOT" >&2
  docker run --rm -v "$SRC_ABS:/workspace:ro" -v "$OUT_DIR:/out" sc-aaa-reproducible-build
else
  echo "space-compute: Docker unavailable, building with the hermetic local toolchain" >&2
  build_core "$SRC_ABS" "$OUT_DIR"
  print_hashes "$OUT_DIR"
fi
