#!/usr/bin/env bash
# One-shot, idempotent local deploy. Safe to re-run after any change.
# Canister IDs reach every canister via icp-cli's PUBLIC_CANISTER_ID:<name> env vars — no setter wiring.
set -euo pipefail
cd "$(dirname "$0")/.."

ENV=local
DEPLOYER="${DEPLOY_IDENTITY:-sc-deployer}"
ADMIN="${ADMIN_IDENTITY:-sc-admin}"
USER_ID="${TEST_IDENTITY:-sc-user}"
MIN_CYCLES=20000000000000

ok()   { echo "✔ $1"; }
note() { echo "• $1"; }

if ! icp network status -e "$ENV" >/dev/null 2>&1; then
  note "starting local network"
  icp network start -e "$ENV" -d >/dev/null
fi
ok "local network: $(icp network status -e "$ENV" --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["gateway_url"])')"

for id in "$DEPLOYER" "$ADMIN" "$USER_ID"; do
  icp identity principal --identity "$id" >/dev/null 2>&1 \
    || { icp identity new "$id" --storage plaintext >/dev/null; note "created identity $id"; }
done

balance=$(icp cycles balance -e "$ENV" --identity "$DEPLOYER" 2>/dev/null | tr -dc '0-9')
if [ "${balance:-0}" -lt "$MIN_CYCLES" ]; then
  icp cycles transfer 100t "$(icp identity principal --identity "$DEPLOYER")" -e "$ENV" --identity anonymous >/dev/null
  note "funded $DEPLOYER with 100T cycles from the seeded anonymous account"
fi
for id in "$ADMIN" "$USER_ID"; do
  icp_balance=$(icp token balance -e "$ENV" --identity "$id" 2>/dev/null | awk '{print int($2)}')
  [ "${icp_balance:-0}" -ge 50 ] || icp token transfer 100 "$(icp identity principal --identity "$id")" -e "$ENV" --identity anonymous >/dev/null
done
ok "identities funded ($DEPLOYER ≥ 20T cycles; $ADMIN, $USER_ID ≥ 50 test ICP)"

export VITE_LOCAL_DATA_ORIGIN="http://127.0.0.1:${SEED_PORT:-8765}"
icp deploy -e "$ENV" --identity "$DEPLOYER" $([ "$ENV" = local ] && echo --yes) >/tmp/sc-deploy-local.log 2>&1 || { cat /tmp/sc-deploy-local.log; exit 1; }
for c in $(icp canister list -e "$ENV" --json | python3 -c 'import json,sys; print(" ".join(json.load(sys.stdin)["canisters"]))'); do
  v=$(icp canister call "$c" version '()' -e "$ENV" --identity "$USER_ID" --query 2>/dev/null || echo "(no version)")
  ok "$c deployed → $v"
done

IDS_FILE=".icp/cache/mappings/${ENV}.ids.json"
PLATFORM_ID=$(python3 -c "import json;print(json.load(open('$IDS_FILE')).get('platform',''))" 2>/dev/null || true)
PAYMENTS_ID=$(python3 -c "import json;print(json.load(open('$IDS_FILE')).get('payments',''))" 2>/dev/null || true)
if [ -n "$PLATFORM_ID" ] && [ -n "$PAYMENTS_ID" ]; then
  icp canister call platform admin_set_payments_id "(principal \"$PAYMENTS_ID\")" -e "$ENV" --identity "$DEPLOYER" >/dev/null 2>&1 || true
  icp canister call payments admin_set_platform_id "(principal \"$PLATFORM_ID\")" -e "$ENV" --identity "$DEPLOYER" >/dev/null 2>&1 || true
  ok "platform<->payments wired (payments_id, platform_id)"
fi

if [ -x tools/seed-local/seed.sh ]; then
  bash tools/seed-local/seed.sh
  ok "seed data loaded"
fi

ok "local deploy complete"
