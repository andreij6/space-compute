#!/usr/bin/env bash
# Cycles + burn + runway per canister, plus treasury health. Exits non-zero if any
# canister's runway is below CYCLES_ALERT_DAYS (default 30), for a cron/launchd monitor.
set -uo pipefail
cd "$(dirname "$0")/.."

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'

ENV="${1:?usage: cycles-report.sh <env>}"
IDENTITY="${DEPLOY_IDENTITY:?set DEPLOY_IDENTITY explicitly (sc-deployer for local)}"
ALERT_DAYS="${CYCLES_ALERT_DAYS:-30}"

STATUS_JSON=$(icp canister status -e "$ENV" --identity "$IDENTITY" --json 2>&1) || { echo "$STATUS_JSON" >&2; exit 1; }

REPORT_PY=$(mktemp)
trap 'rm -f "$REPORT_PY"' EXIT
cat > "$REPORT_PY" <<'PY'
import json, sys
alert_days = int(sys.argv[1])
worst_ok = True
print("{:<12}{:>12}{:>14}{:>14}".format("canister", "cycles(T)", "burn/day(T)", "runway(days)"))
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    c = json.loads(line)
    cycles = int(c['cycles'])
    burn = int(c['idle_cycles_burned_per_day'])
    runway = (cycles // burn) if burn > 0 else -1
    runway_s = str(runway) if runway >= 0 else 'inf'
    print("{:<12}{:>12.2f}{:>14.4f}{:>14}".format(c['name'], cycles / 1e12, burn / 1e12, runway_s))
    if 0 <= runway < alert_days:
        worst_ok = False
sys.exit(0 if worst_ok else 1)
PY
echo "$STATUS_JSON" | python3 "$REPORT_PY" "$ALERT_DAYS"
CANISTER_ALERT=$?

echo
HEALTH=$(icp canister call treasury health '()' -e "$ENV" --identity "$IDENTITY" --query 2>&1) || { echo "$HEALTH" >&2; exit 1; }
echo "treasury.health():"
echo "$HEALTH" | sed 's/^/  /'
echo "$HEALTH" | grep -q 'reserve_breached = true' && RESERVE_BREACHED=1 || RESERVE_BREACHED=0

if [ "$CANISTER_ALERT" != "0" ] || [ "$RESERVE_BREACHED" = "1" ]; then
  echo -e "${RED}✘${NC} cycles alert: a canister is under ${ALERT_DAYS}d runway or the treasury reserve is breached" >&2
  exit 1
fi
echo -e "${GREEN}✔${NC} all canisters above ${ALERT_DAYS}d runway, reserve not breached"
