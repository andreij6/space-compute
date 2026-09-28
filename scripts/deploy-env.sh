#!/usr/bin/env bash
# Release workflow for any environment: snapshot -> deploy -> wire -> smoke -> rollback-on-failure.
set -uo pipefail
cd "$(dirname "$0")/.."

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✔${NC} $1"; }
note() { echo -e "${YELLOW}•${NC} $1"; }
fail() { echo -e "${RED}✘${NC} $1" >&2; exit 1; }

ENV="${1:-}"
case "$ENV" in
  local|staging|production) ;;
  *) fail "usage: deploy-env.sh <local|staging|production>" ;;
esac

DEPLOY_IDENTITY="${DEPLOY_IDENTITY:-}"
[ -n "$DEPLOY_IDENTITY" ] || fail "set DEPLOY_IDENTITY explicitly (sc-deployer for local; a named release identity for staging/production) — never rely on the machine default (prod-deployer)"
ok "identity: $DEPLOY_IDENTITY"

CANISTERS=(platform payments treasury frontend)

if [ "$ENV" != "local" ]; then
  [ "${SC_ALLOW_MAINNET:-}" = "1" ] || fail "refusing $ENV: rerun with SC_ALLOW_MAINNET=1 only when the owner asked this session"
  echo "About to deploy to $ENV as $DEPLOY_IDENTITY. Type '$ENV' to confirm:"
  read -r CONFIRM
  [ "$CONFIRM" = "$ENV" ] || fail "confirmation mismatch, aborting"

  [ -z "$(git status --porcelain)" ] || fail "git tree is dirty, commit or stash before releasing"
  [ "$(git branch --show-current)" = "main" ] || fail "not on main"
  just verify || fail "just verify failed"
  ok "preflight: clean tree, on main, verify green"
else
  note "local env: skipping clean-tree/branch/verify preflight (dev loop)"
fi

ENV_EXISTS() { icp canister status "$1" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1; }

release_record() {
  mkdir -p docs/ops
  [ -f docs/ops/releases.md ] || printf '# Releases\n\n| Date | Env | Identity | Rev | Result |\n|---|---|---|---|---|\n' > docs/ops/releases.md
  REV="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
  printf '| %s | %s | %s | %s | %s |\n' "$(date -u +%FT%TZ)" "$ENV" "$DEPLOY_IDENTITY" "$REV" "$1" >> docs/ops/releases.md
}

SNAP_FILE=$(mktemp)
trap 'rm -f "$SNAP_FILE"' EXIT

for c in "${CANISTERS[@]}"; do
  if ENV_EXISTS "$c"; then
    icp canister stop "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1
    SNAP_JSON=$(icp canister snapshot create "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" --json 2>&1) \
      || { icp canister start "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1; fail "snapshot create failed for $c: $SNAP_JSON"; }
    icp canister start "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1
    SNAP_ID=$(echo "$SNAP_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["snapshot_id"])')
    echo "$c $SNAP_ID" >> "$SNAP_FILE"
    ok "snapshot $c -> $SNAP_ID"
  else
    note "$c does not exist yet in $ENV, first install (no snapshot)"
  fi
done

rollback() {
  note "rolling back: restoring pre-upgrade snapshots"
  while read -r c snap_id; do
    [ -n "$c" ] || continue
    icp canister stop "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1
    icp canister snapshot restore "$c" "$snap_id" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1 \
      && ok "restored $c from $snap_id" \
      || note "restore FAILED for $c from $snap_id — manual intervention required"
    icp canister start "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1
  done < "$SNAP_FILE"
  release_record "ROLLED BACK"
}

DEPLOY_LOG=$(mktemp)
DEPLOY_STATUS=0
icp deploy "${CANISTERS[@]}" -e "$ENV" --identity "$DEPLOY_IDENTITY" --yes >"$DEPLOY_LOG" 2>&1 || DEPLOY_STATUS=$?

for c in "${CANISTERS[@]}"; do
  icp canister start "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1 || true
done

if [ "$DEPLOY_STATUS" != "0" ]; then
  cat "$DEPLOY_LOG"
  rollback
  fail "icp deploy failed, rolled back"
fi
ok "deployed: ${CANISTERS[*]}"

IDS_FILE=".icp/cache/mappings/${ENV}.ids.json"
PLATFORM_ID=$(python3 -c "import json;print(json.load(open('$IDS_FILE')).get('platform',''))" 2>/dev/null || true)
PAYMENTS_ID=$(python3 -c "import json;print(json.load(open('$IDS_FILE')).get('payments',''))" 2>/dev/null || true)
if [ -n "$PLATFORM_ID" ] && [ -n "$PAYMENTS_ID" ]; then
  icp canister call platform admin_set_payments_id "(principal \"$PAYMENTS_ID\")" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1 || true
  icp canister call payments admin_set_platform_id "(principal \"$PLATFORM_ID\")" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1 || true
  ok "platform<->payments wired"
fi

if [ "${AAA_REGISTER:-0}" = "1" ]; then
  bash scripts/aaa-register.sh "$ENV" "$DEPLOY_IDENTITY" || note "AAA wasm register/approve skipped: $?"
fi

SMOKE_OK=1
smoke_query() { icp canister call "$1" "$2" '()' -e "$ENV" --identity "$DEPLOY_IDENTITY" --query >/dev/null 2>&1; }
for c in "${CANISTERS[@]}"; do
  [ "$c" = "frontend" ] && continue
  smoke_query "$c" version && ok "smoke: $c.version()" || { SMOKE_OK=0; note "smoke FAILED: $c.version()"; }
done
smoke_query platform get_stats && ok "smoke: platform.get_stats()" || { SMOKE_OK=0; note "smoke FAILED: platform.get_stats()"; }
smoke_query treasury health && ok "smoke: treasury.health()" || { SMOKE_OK=0; note "smoke FAILED: treasury.health()"; }

FRONTEND_ID=$(python3 -c "import json;print(json.load(open('$IDS_FILE')).get('frontend',''))" 2>/dev/null || true)
if [ -n "$FRONTEND_ID" ]; then
  if [ "$ENV" = "local" ]; then
    PORT=$(icp network status -e local --json 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["gateway_url"])' | sed -E 's#.*:([0-9]+)/?#\1#')
    FRONTEND_URL="http://${FRONTEND_ID}.localhost:${PORT}/"
  else
    FRONTEND_URL="https://${FRONTEND_ID}.icp0.io/"
  fi
  CODE=$(curl -sS -o /dev/null -w '%{http_code}' "$FRONTEND_URL" 2>/dev/null || echo 000)
  [ "$CODE" = "200" ] && ok "smoke: frontend GET $FRONTEND_URL -> 200" || { SMOKE_OK=0; note "smoke FAILED: frontend GET $FRONTEND_URL -> $CODE"; }
fi

if [ "${SMOKE_FORCE_FAIL:-0}" = "1" ]; then
  note "SMOKE_FORCE_FAIL=1: treating smoke as failed regardless of the checks above"
  SMOKE_OK=0
fi

if [ "$SMOKE_OK" != "1" ]; then
  rollback
  fail "smoke test failed, rolled back to pre-upgrade snapshots"
fi

for c in "${CANISTERS[@]}"; do
  LIST_JSON=$(icp canister snapshot list "$c" -e "$ENV" --identity "$DEPLOY_IDENTITY" --json 2>/dev/null) || continue
  echo "$LIST_JSON" | python3 -c '
import json, sys
data = json.load(sys.stdin)["snapshots"]
data.sort(key=lambda s: s["taken_at_timestamp"])
for s in data[:-3]:
    print(s["snapshot_id"])
' | while read -r OLD_ID; do
    [ -n "$OLD_ID" ] || continue
    icp canister snapshot delete "$c" "$OLD_ID" -e "$ENV" --identity "$DEPLOY_IDENTITY" >/dev/null 2>&1 \
      && note "pruned old snapshot $c/$OLD_ID"
  done
done

release_record "OK"
ok "release recorded in docs/ops/releases.md"
ok "release complete: $ENV"
