#!/usr/bin/env bash
# Builds the vN-1 wasms used by the t7_1_* upgrade tests, one pinned commit per canister
# (never a moving ref like HEAD~1). Each baseline is the latest commit whose crate source
# differs from the current tree with a real stored-state change in between, so the test is
# genuinely cross-version (the tests also assert baseline module hash != current):
#   platform 992a48c  T4.6 done; events/progression/registry state has changed since.
#   payments 25c786a  T5.15, before T5.7: later commits add the AAA/owner op indexes and
#                     reshape invites and the journal.
#   treasury d31c4a5  T5.17, the only earlier treasury with state; later source differs
#                     (post_upgrade admin bootstrap). No committed treasury change after it
#                     touches stored types; move this forward once the proposals rework lands.
#   aaa      cbbcc90  T3.5, before T3.4 added the burn EMA/heartbeat/credits timer state.
# Output is cached under target/upgrade-baseline/<commit>/ (gitignored) and the chosen set is
# linked at target/upgrade-baseline/<canister>.wasm.gz. Bump a commit at each release once a
# newer vN exists to diff against; the script refuses a baseline whose source equals the tree.
set -euo pipefail

BASELINES=(
  "platform 992a48c443d1e99ab8d6384c54fdb6e4576a60e8"
  "payments 25c786a09fbc14f95f79e1484abb3b36941fca6d"
  "treasury d31c4a549dd5110d69c0465217a2b55656f14e75"
  "aaa cbbcc904e571f7959d252402757daeedbd1db226"
)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/target/upgrade-baseline"
WORKTREE="$ROOT/target/upgrade-baseline-worktree"
trap 'git -C "$ROOT" worktree remove --force "$WORKTREE" >/dev/null 2>&1 || true' EXIT

for entry in "${BASELINES[@]}"; do
  read -r name commit <<<"$entry"
  if git -C "$ROOT" diff --quiet "$commit" -- "crates/$name/src"; then
    echo "baseline $commit has the same crates/$name/src as the tree: not a cross-version test" >&2
    exit 1
  fi
  cached="$OUT/$commit/$name.wasm.gz"
  if [ ! -f "$cached" ]; then
    git -C "$ROOT" worktree remove --force "$WORKTREE" >/dev/null 2>&1 || rm -rf "$WORKTREE"
    git -C "$ROOT" worktree add --detach --quiet "$WORKTREE" "$commit"
    (cd "$WORKTREE" && CARGO_TARGET_DIR="$OUT/target" cargo build -q --target wasm32-unknown-unknown --release -p "$name")
    mkdir -p "$OUT/$commit"
    gzip -9 -c "$OUT/target/wasm32-unknown-unknown/release/${name}.wasm" > "$cached"
  fi
  ln -sfn "$commit/$name.wasm.gz" "$OUT/$name.wasm.gz"
done
