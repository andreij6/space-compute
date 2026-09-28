#!/usr/bin/env bash
set -uo pipefail

RUNNER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${SPACE_COMPUTE_DIR:-$PWD}"
CONFIG_FILE="$PROJECT_DIR/.space-compute.json"
DID="$RUNNER_DIR/../skills/space-compute-astronomer/reference/aaa.did"
LOG_DIR="$HOME/.space-compute/logs"
THRESHOLD_DAYS="${SPACE_COMPUTE_FUEL_THRESHOLD:-3}"
BATCH_SIZE="${SPACE_COMPUTE_BATCH_SIZE:-20}"
MAX_TURNS="${SPACE_COMPUTE_MAX_TURNS:-300}"
MAX_BUDGET_USD="${SPACE_COMPUTE_MAX_BUDGET_USD:-5}"

mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/runner-$(date +%Y%m%d).log"
log() { echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $1" | tee -a "$LOG_FILE"; }

if [ ! -f "$CONFIG_FILE" ]; then
  log "error: missing $CONFIG_FILE"
  exit 1
fi

cfg() {
  python3 -c 'import json,sys;print(json.load(open(sys.argv[1])).get(sys.argv[2],sys.argv[3]))' "$CONFIG_FILE" "$1" "$2"
}
AAA_ID=$(cfg aaa "")
IDENTITY=$(cfg identity "")
NET=$(cfg net "-n ic")

case "$NET" in
  "-n ic") NET_ARGS=(-n ic) ;;
  "-e local") NET_ARGS=(-e local) ;;
  *) log "error: net must be \"-n ic\" or \"-e local\""; exit 1 ;;
esac
if ! [[ "$AAA_ID" =~ ^([a-z2-7]{5}-){1,12}[a-z2-7]{1,5}$ ]]; then
  log "error: aaa is not a principal"
  exit 1
fi
if ! [[ "$IDENTITY" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  log "error: identity must match ^[a-zA-Z0-9_-]+$"
  exit 1
fi

STATUS_JSON=$(icp canister call "$AAA_ID" status '()' "${NET_ARGS[@]}" --identity "$IDENTITY" --query --candid "$DID" --json 2>&1)
if [ $? -ne 0 ]; then
  log "error: icp status call failed: $STATUS_JSON"
  exit 1
fi

DAYS=$(printf '%s' "$STATUS_JSON" | python3 -c '
import json, math, re, sys
text = json.load(sys.stdin).get("response_candid") or ""
m = re.search(r"days_of_fuel_estimate = (-?[0-9a-z_.+]+)", text)
if "Ok =" not in text or not m:
    sys.exit(1)
days = float(m.group(1).replace("_", ""))
if math.isnan(days):
    sys.exit(1)
print(days)
' 2>/dev/null)
if [ -z "$DAYS" ]; then
  log "error: status did not return Ok with days_of_fuel_estimate: $STATUS_JSON"
  exit 1
fi

GUARD_OUT=$(python3 "$RUNNER_DIR/guard.py" "$DAYS" "$THRESHOLD_DAYS")
GUARD_STATUS=$?
if [ $GUARD_STATUS -ne 0 ]; then
  log "$GUARD_OUT"
  exit 0
fi

log "fuel guard passed: $DAYS days >= threshold $THRESHOLD_DAYS, running batch of $BATCH_SIZE"
cd "$PROJECT_DIR" || exit 1
claude -p "run $BATCH_SIZE Space Compute tasks" \
  --tools Bash Read \
  --allowedTools "Bash(icp canister call:*)" "Bash(curl -sfo work/:*)" "Bash(shasum -a 256:*)" "Bash(mkdir -p work/:*)" Read \
  --max-turns "$MAX_TURNS" \
  --max-budget-usd "$MAX_BUDGET_USD" >>"$LOG_FILE" 2>&1
log "run complete, exit=$?"
