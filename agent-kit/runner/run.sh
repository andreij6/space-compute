#!/usr/bin/env bash
set -uo pipefail

RUNNER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${SPACE_COMPUTE_DIR:-$PWD}"
CONFIG_FILE="$PROJECT_DIR/.space-compute.json"
LOG_DIR="$HOME/.space-compute/logs"
THRESHOLD_DAYS="${SPACE_COMPUTE_FUEL_THRESHOLD:-3}"
BATCH_SIZE="${SPACE_COMPUTE_BATCH_SIZE:-20}"

mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/runner-$(date +%Y%m%d).log"
log() { echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $1" | tee -a "$LOG_FILE"; }

if [ ! -f "$CONFIG_FILE" ]; then
  log "error: missing $CONFIG_FILE"
  exit 1
fi

AAA_ID=$(python3 -c "import json;print(json.load(open('$CONFIG_FILE'))['aaa'])")
IDENTITY=$(python3 -c "import json;print(json.load(open('$CONFIG_FILE'))['identity'])")
NET=$(python3 -c "import json;print(json.load(open('$CONFIG_FILE')).get('net','-n ic'))")

STATUS_JSON=$(icp canister call "$AAA_ID" status '()' $NET --identity "$IDENTITY" --query --json 2>&1)
if [ $? -ne 0 ]; then
  log "error: icp status call failed: $STATUS_JSON"
  exit 1
fi

DAYS=$(STATUS_JSON="$STATUS_JSON" python3 -c "
import json, os, sys
try:
    print(json.loads(os.environ['STATUS_JSON'])['days_of_fuel_estimate'])
except Exception:
    sys.exit(1)
" 2>/dev/null)
if [ -z "$DAYS" ]; then
  log "error: could not parse days_of_fuel_estimate from status output"
  exit 1
fi

GUARD_OUT=$(python3 "$RUNNER_DIR/guard.py" "$DAYS" "$THRESHOLD_DAYS")
GUARD_STATUS=$?
if [ $GUARD_STATUS -ne 0 ]; then
  log "$GUARD_OUT"
  exit 0
fi

log "fuel guard passed: $DAYS days >= threshold $THRESHOLD_DAYS, running batch of $BATCH_SIZE"
claude -p "run $BATCH_SIZE Space Compute tasks" >>"$LOG_FILE" 2>&1
log "run complete, exit=$?"
