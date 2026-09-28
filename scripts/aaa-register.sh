#!/usr/bin/env bash
# Registers and approves the AAA template wasm on `platform` if the reproducible build's
# hash differs from what's already approved. Opt-in (deploy-env.sh AAA_REGISTER=1) since
# the reproducible build is slow; run standalone any time to check/update.
set -uo pipefail
cd "$(dirname "$0")/.."

ENV="${1:?usage: aaa-register.sh <env> <identity>}"
IDENTITY="${2:?usage: aaa-register.sh <env> <identity>}"

OUT_DIR="target/aaa-reproducible/${ENV}"
BUILD_LOG=$(bash scripts/build-aaa-reproducible.sh . "$OUT_DIR" 2>&1) || { echo "$BUILD_LOG" >&2; exit 1; }
MODULE_SHA256=$(echo "$BUILD_LOG" | sed -nE 's/^module_sha256=(.*)/\1/p')
GZ_SHA256=$(echo "$BUILD_LOG" | sed -nE 's/^gz_sha256=(.*)/\1/p')
[ -n "$MODULE_SHA256" ] || { echo "could not determine module_sha256" >&2; exit 1; }

LIST=$(icp canister call platform admin_list_wasms '()' -e "$ENV" --identity "$IDENTITY" --query 2>&1) \
  || { echo "$LIST" >&2; exit 1; }

PARSE_PY=$(mktemp)
trap 'rm -f "$PARSE_PY"' EXIT
cat > "$PARSE_PY" <<'PY'
import re, sys
target = sys.argv[1].lower()
text = sys.stdin.read()
entries = re.findall(r'(\d+)\s*:\s*nat32;\s*record\s*\{[^}]*?module_sha256\s*=\s*blob\s*"((?:\\[0-9a-f]{2})+)"[^}]*?approved\s*=\s*(true|false)', text, re.S)
max_v = 0
for v, blob, approved in entries:
    v = int(v)
    max_v = max(max_v, v)
    h = blob.replace('\\', '')
    if h.lower() == target and approved == 'true':
        print("ALREADY_APPROVED")
        sys.exit(0)
print(f"NEXT_VERSION={max_v + 1}")
PY
DECISION=$(printf '%s' "$LIST" | python3 "$PARSE_PY" "$MODULE_SHA256")

if [ "$DECISION" = "ALREADY_APPROVED" ]; then
  echo "AAA wasm already registered and approved (module_sha256=$MODULE_SHA256)"
  exit 0
fi

VERSION="${DECISION#NEXT_VERSION=}"
GZ_FILE="$OUT_DIR/aaa.wasm.gz"
ARGS_FILE=$(mktemp)
python3 - "$VERSION" "$GZ_FILE" "$GZ_SHA256" "$ARGS_FILE" <<'PY'
import sys
version, gz_path, gz_sha256_hex, out_path = sys.argv[1:5]
data = open(gz_path, "rb").read()
sha = bytes.fromhex(gz_sha256_hex)
def esc(b): return "".join(f"\\{x:02x}" for x in b)
with open(out_path, "w") as f:
    f.write(f'({version} : nat32, blob "{esc(data)}", blob "{esc(sha)}")')
PY

icp canister call platform admin_upload_wasm --args-file "$ARGS_FILE" --args-format candid -e "$ENV" --identity "$IDENTITY" >/dev/null
rm -f "$ARGS_FILE"
icp canister call platform admin_approve_wasm "($VERSION : nat32)" -e "$ENV" --identity "$IDENTITY" >/dev/null
echo "AAA wasm v$VERSION registered and approved (module_sha256=$MODULE_SHA256)"
