#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
ID=(--identity sc-deployer -e local)
PORT="${SEED_PORT:-8765}"
LIMIT="${SEED_SUBJECTS:-500}"
BUCKET=target/bucket

if [ -d "$BUCKET" ] && ! curl -s -o /dev/null "http://127.0.0.1:$PORT/manifest_v1.jsonl"; then
  nohup python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$BUCKET" </dev/null >/tmp/sc-bucket-http.log 2>&1 &
  echo $! > target/seed-http.pid
  disown
  sleep 1
fi
[ -d "$BUCKET" ] || echo "  note: $BUCKET missing (run just curate-dossiers); subjects are seeded but images won't load"

cargo build -q --target wasm32-unknown-unknown --release -p aaa
cargo run -q -p integration-tests --bin seed_args -- "http://127.0.0.1:$PORT" "$LIMIT"

call() {
  local out
  out=$(icp canister call platform "$@" "${ID[@]}" 2>&1) || { echo "$out"; exit 1; }
  if grep -q "Err" <<<"$out" && ! grep -q "Conflict" <<<"$out"; then
    echo "  seed call failed: $1 → $out"
    exit 1
  fi
}

call admin_add_protocol --args-file target/seed/protocol.bin --args-format bin
call admin_set_current_protocol '(1 : nat16)'
for f in target/seed/subjects_*.bin; do
  call admin_add_subjects --args-file "$f" --args-format bin
done
call admin_upload_wasm --args-file target/seed/aaa_wasm.bin --args-format bin
call admin_approve_wasm '(1 : nat32)'

call admin_add_honeypots --args-file target/seed/honeypots.bin --args-format bin
echo "  seeded protocol v1, $LIMIT subjects (images at http://127.0.0.1:$PORT), AAA wasm v1, honeypots"
