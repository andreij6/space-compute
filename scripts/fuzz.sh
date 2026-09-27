#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../fuzz"
SECS="${1:-60}"
for t in decode_submission validate_text; do
  echo "fuzzing $t for ${SECS}s"
  cargo +nightly fuzz run "$t" -- -max_total_time="$SECS" -rss_limit_mb=2048 2>&1 | grep -E "Done|SUMMARY|crash|ERROR" || true
  [ -z "$(ls artifacts/$t 2>/dev/null)" ] || { echo "crash artifacts in fuzz/artifacts/$t"; exit 1; }
done
