#!/usr/bin/env bash
# Proves "hash reproducible on 2 machines" (08 §7 / T7.4): builds crates/aaa
# twice, in two unrelated absolute directories with different env, and
# fails if the module or gzip hashes disagree. With a running Docker daemon
# machine A is the pinned container build and machine B the native hermetic
# build (a real cross-toolchain comparison); without Docker both are native
# and the script says so. Owners run this same script
# to self-verify before trusting a published sha256 (docs/ops/aaa-manual-upgrade.md).
# Copies the working tree (not a `git worktree`) into each directory, since a
# `git worktree` only sees committed content and this repo has other lanes
# mid-commit; copying the tracked files as they stand on disk keeps this
# check independent of that churn while still isolating each build's path.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

STAMP="$$-$RANDOM"
WT_A="/tmp/sc-aaa-repro-$STAMP/machine-a"
WT_B="/tmp/sc-aaa-repro-$STAMP/machine-b/deeply/nested/checkout"
OUT_A="$ROOT/target/aaa-reproducible/check-a"
OUT_B="$ROOT/target/aaa-reproducible/check-b"

cleanup() { rm -rf "/tmp/sc-aaa-repro-$STAMP"; }
trap cleanup EXIT

copy_tree() {
  mkdir -p "$1"
  cp "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$ROOT/rust-toolchain.toml" "$ROOT/Dockerfile.aaa-reproducible" "$1/"
  cp -R "$ROOT/crates" "$1/crates"
  cp -R "$ROOT/scripts" "$1/scripts"
}

copy_tree "$WT_A"
copy_tree "$WT_B"

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  MODE_A=docker
  echo "space-compute: Docker available: comparing a container build (A) against a native build (B)"
else
  MODE_A=local
  echo "space-compute: Docker unavailable: comparing two native builds only (no container-vs-native evidence)"
fi

echo "space-compute: machine A ($MODE_A) -> $WT_A"
HASHES_A="$(AAA_BUILD_MODE=$MODE_A USER=sc-owner-a LANG=en_US.UTF-8 bash "$WT_A/scripts/build-aaa-reproducible.sh" "$WT_A" "$OUT_A")"
echo "$HASHES_A"

echo "space-compute: machine B (local) -> $WT_B"
HASHES_B="$(AAA_BUILD_MODE=local USER=sc-owner-b LANG=C bash "$WT_B/scripts/build-aaa-reproducible.sh" "$WT_B" "$OUT_B")"
echo "$HASHES_B"

if [ "$HASHES_A" != "$HASHES_B" ]; then
  echo "MISMATCH: the two builds produced different hashes" >&2
  exit 1
fi
echo "space-compute: reproducible: identical hashes from two independent build directories ($MODE_A vs local)"
