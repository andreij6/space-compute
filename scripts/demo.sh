#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/.."
TASK="$1"
PREFIX="$(echo "$TASK" | tr 'A-Z.-' 'a-z__')_"
echo "▶ demo $TASK (tests matching ${PREFIX}*)"
OUT=$(POCKET_IC_MUTE_SERVER=1 cargo test --workspace -q -- "$PREFIX" --nocapture --test-threads=1 2>&1)
STATUS=$?
echo "$OUT" | grep -vE '^$|running 0 tests|0 passed; 0 failed; 0 ignored; 0 measured|^\.+$' | sed 's/^\.\+//'
PASSED=$(echo "$OUT" | grep -oE '[0-9]+ passed' | awk '{s+=$1} END {print s+0}')
if [ "$STATUS" -ne 0 ]; then echo "✗ demo failed"; exit 1; fi
if [ "$PASSED" -eq 0 ]; then echo "✗ no tests named ${PREFIX}* — a demo with no tests is a failure"; exit 1; fi
echo "✔ $PASSED tests passed for $TASK"
