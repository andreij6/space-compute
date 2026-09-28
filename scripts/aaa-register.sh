#!/usr/bin/env bash
# Registers and approves the AAA template wasm on `platform` if the reproducible build's
# hash differs from what's already approved. Opt-in (deploy-env.sh AAA_REGISTER=1) since
# the reproducible build is slow; run standalone any time to check/update.
# Production additionally requires the same module to have been approved on staging >= 48 h
# ago (09-testing-ops.md, Release: staging soak); SOAK_IDENTITY names the staging identity.
set -uo pipefail
cd "$(dirname "$0")/.."
source scripts/icp-guard.sh

ENV="${1:?usage: aaa-register.sh <env> <identity>}"
IDENTITY="${2:?usage: aaa-register.sh <env> <identity>}"
mainnet_guard "$ENV" "$IDENTITY"

OUT_DIR="target/aaa-reproducible/${ENV}"
BUILD_LOG=$(bash scripts/build-aaa-reproducible.sh . "$OUT_DIR" 2>&1) || guard_die "reproducible build failed: $BUILD_LOG"
MODULE_SHA256=$(echo "$BUILD_LOG" | sed -nE 's/^module_sha256=(.*)/\1/p')
GZ_SHA256=$(echo "$BUILD_LOG" | sed -nE 's/^gz_sha256=(.*)/\1/p')
[ -n "$MODULE_SHA256" ] && [ -n "$GZ_SHA256" ] || guard_die "could not determine module_sha256/gz_sha256"

if [ "$ENV" = "production" ]; then
  SOAK_ENV="${SOAK_ENV:-staging}"
  SOAK_LIST=$(icp_must canister call platform admin_list_wasms '()' -e "$SOAK_ENV" --identity "${SOAK_IDENTITY:-$IDENTITY}" --query) || exit 1
  printf '%s' "$SOAK_LIST" | python3 scripts/aaa-wasms.py soaked "$MODULE_SHA256" \
    || guard_die "module $MODULE_SHA256 has not soaked >= 48 h on $SOAK_ENV (approve it there first)"
  echo "$SOAK_ENV soak >= 48 h confirmed for $MODULE_SHA256"
fi

LIST=$(icp_must canister call platform admin_list_wasms '()' -e "$ENV" --identity "$IDENTITY" --query) || exit 1
DECISION=$(printf '%s' "$LIST" | python3 scripts/aaa-wasms.py decide "$MODULE_SHA256")

if [ "$DECISION" = "ALREADY_APPROVED" ]; then
  echo "AAA wasm already registered and approved (module_sha256=$MODULE_SHA256)"
  exit 0
fi

VERSION="${DECISION#NEXT_VERSION=}"
GZ_FILE="$OUT_DIR/aaa.wasm.gz"
ARGS_FILE=$(mktemp)
trap 'rm -f "$ARGS_FILE"' EXIT
python3 - "$VERSION" "$GZ_FILE" "$GZ_SHA256" "$ARGS_FILE" <<'PY'
import sys
version, gz_path, gz_sha256_hex, out_path = sys.argv[1:5]
data = open(gz_path, "rb").read()
sha = bytes.fromhex(gz_sha256_hex)
def esc(b): return "".join(f"\\{x:02x}" for x in b)
with open(out_path, "w") as f:
    f.write(f'({version} : nat32, blob "{esc(data)}", blob "{esc(sha)}")')
PY

icp_must canister call platform admin_upload_wasm --args-file "$ARGS_FILE" --args-format candid -e "$ENV" --identity "$IDENTITY" >/dev/null
icp_must canister call platform admin_approve_wasm "($VERSION : nat32)" -e "$ENV" --identity "$IDENTITY" >/dev/null
echo "AAA wasm v$VERSION registered and approved (module_sha256=$MODULE_SHA256)"
