#!/usr/bin/env bash
# Proves "hash reproducible on 2 machines" (08 §7 / T7.4): builds crates/aaa
# twice, in two unrelated absolute directories with different env, and
# fails if the module or gzip hashes disagree. Owners run this same script
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
  cp "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$ROOT/rust-toolchain.toml" "$1/"
  cp -R "$ROOT/crates" "$1/crates"
}

copy_tree "$WT_A"
copy_tree "$WT_B"

echo "space-compute: machine A -> $WT_A"
HASHES_A="$(USER=sc-owner-a LANG=en_US.UTF-8 bash "$ROOT/scripts/build-aaa-reproducible.sh" "$WT_A" "$OUT_A")"
echo "$HASHES_A"

echo "space-compute: machine B -> $WT_B"
HASHES_B="$(USER=sc-owner-b LANG=C bash "$ROOT/scripts/build-aaa-reproducible.sh" "$WT_B" "$OUT_B")"
echo "$HASHES_B"

if [ "$HASHES_A" != "$HASHES_B" ]; then
  echo "MISMATCH: the two builds produced different hashes" >&2
  exit 1
fi
echo "space-compute: reproducible — identical hashes from two independent build directories"
