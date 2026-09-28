#!/usr/bin/env bash
# Adds platform/payments/frontend/treasury to the treasury watch list (12-treasury-keeper.md §2)
# and makes treasury a co-controller of platform/payments/frontend (needed for canister_status).
set -uo pipefail
cd "$(dirname "$0")/.."
source scripts/icp-guard.sh

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✔${NC} $1"; }
note() { echo -e "${YELLOW}•${NC} $1"; }

ENV="${1:?usage: treasury-watch.sh <env>}"
IDENTITY="${DEPLOY_IDENTITY:-}"
mainnet_guard "$ENV" "$IDENTITY"

STATUS_JSON=$(icp canister status -e "$ENV" --identity "$IDENTITY" --json 2>&1) || guard_die "canister status failed: $STATUS_JSON"

get_field() { echo "$STATUS_JSON" | python3 -c "
import json, sys
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    c = json.loads(line)
    if c['name'] == '$1':
        print(c['$2'])
"; }

TREASURY_ID=$(get_field treasury id)
[ -n "$TREASURY_ID" ] || guard_die "treasury canister not found in $ENV"

TARGET_DAYS=60

priority_of() {
  case "$1" in
    platform) echo 1 ;;
    payments) echo 2 ;;
    frontend) echo 3 ;;
    treasury) echo 4 ;;
  esac
}

for name in platform payments frontend treasury; do
  ID=$(get_field "$name" id)
  [ -n "$ID" ] || { note "$name not found in $ENV, skipping watch"; continue; }
  PRIO=$(priority_of "$name")
  icp_must canister call treasury admin_watch "(principal \"$ID\", ${PRIO} : nat8, ${TARGET_DAYS} : nat32)" \
    -e "$ENV" --identity "$IDENTITY" >/dev/null
  ok "watching $name ($ID) priority=${PRIO} target_days=${TARGET_DAYS}"
done

for name in platform payments frontend; do
  CONTROLLERS=$(echo "$STATUS_JSON" | python3 -c "
import json, sys
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    c = json.loads(line)
    if c['name'] == '$name':
        print(' '.join(c['settings']['controllers']))
")
  if echo "$CONTROLLERS" | grep -q "$TREASURY_ID"; then
    note "$name already has treasury ($TREASURY_ID) as a controller"
  else
    icp_must canister settings update "$name" --add-controller "$TREASURY_ID" -e "$ENV" --identity "$IDENTITY" -f >/dev/null
    ok "added treasury as a controller of $name"
  fi
done

echo
echo "treasury.status() watch list:"
icp_must canister call treasury status '()' -e "$ENV" --identity "$IDENTITY" --query | sed 's/^/  /'
