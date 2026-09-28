#!/usr/bin/env bash
# Builds the vN-1 wasms used by the t7_1_* upgrade tests from a fixed, pinned commit
# (never a moving ref like HEAD~1). Output is cached under target/, which is gitignored,
# so nothing binary is committed. Re-run after bumping BASELINE_COMMIT to a newer
# released commit; do that at each release once a real vN exists to diff against.
# Picked 992a48c (T4.6 done) over HEAD: HEAD at T7.1 authoring time did not build
# standalone (progression.rs referenced AaaRecord.is_house before that field existed
# in a committed registry.rs; fixed later by the T4.10/T4.11 lane).
set -euo pipefail

BASELINE_COMMIT="992a48c443d1e99ab8d6384c54fdb6e4576a60e8"
CANISTERS=(platform payments treasury aaa)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/upgrade-baseline/$BASELINE_COMMIT"

if [ -f "$CACHE/.done" ]; then
  exit 0
fi

WORKTREE="$ROOT/target/upgrade-baseline-worktree"
rm -rf "$WORKTREE"
git -C "$ROOT" worktree add --detach --quiet "$WORKTREE" "$BASELINE_COMMIT"
trap 'git -C "$ROOT" worktree remove --force "$WORKTREE" >/dev/null 2>&1 || true' EXIT

mkdir -p "$CACHE"
(
  cd "$WORKTREE"
  args=()
  for name in "${CANISTERS[@]}"; do
    args+=(-p "$name")
  done
  cargo build -q --target wasm32-unknown-unknown --release "${args[@]}"
)

for name in "${CANISTERS[@]}"; do
  gzip -9 -c "$WORKTREE/target/wasm32-unknown-unknown/release/${name}.wasm" \
    > "$CACHE/${name}.wasm.gz"
done
touch "$CACHE/.done"
