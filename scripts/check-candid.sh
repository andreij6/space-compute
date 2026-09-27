#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
DID_ROOT="${DID_ROOT:-crates}"
MODE="${1:-check}"
CANISTERS=(platform payments treasury aaa)
cargo build -q --target wasm32-unknown-unknown --release $(printf -- '-p %s ' "${CANISTERS[@]}")
drift=0
for c in "${CANISTERS[@]}"; do
  fresh=$(candid-extractor "target/wasm32-unknown-unknown/release/$c.wasm")
  committed="$DID_ROOT/$c/$c.did"
  if [ "$MODE" = "--write" ]; then
    printf '%s\n' "$fresh" > "crates/$c/$c.did"
    echo "wrote crates/$c/$c.did"
  elif ! diff -q <(printf '%s\n' "$fresh") "$committed" >/dev/null 2>&1; then
    echo "candid drift: $committed differs from the code (run: just candid)"
    diff <(printf '%s\n' "$fresh") "$committed" | head -20 || true
    drift=1
  fi
done
exit $drift
