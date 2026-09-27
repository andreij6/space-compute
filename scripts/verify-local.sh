#!/usr/bin/env bash
# Local verification suite (no remote CI). Every gate must pass before a task is committed to main.
set -uo pipefail
cd "$(dirname "$0")/.."

GREEN='\033[0;32m'; RED='\033[0;31m'; BLUE='\033[0;34m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✔${NC} $1"; }
note() { echo -e "${YELLOW}•${NC} $1"; }
fail() { echo -e "${RED}✘${NC} $1"; exit 1; }

echo -e "${BLUE}Space Compute — local verification${NC}"

cargo fmt --all -- --check >/dev/null 2>&1 || fail "cargo fmt: run 'just fmt'"
ok "fmt"

cargo clippy -q --all-targets -- -D warnings 2>&1 | tail -20
[ "${PIPESTATUS[0]}" -eq 0 ] || fail "clippy warnings"
ok "clippy (-D warnings)"

bash scripts/check-candid.sh || fail "candid drift"
ok "candid .did files match the code"

AAA_WASM="target/wasm32-unknown-unknown/release/aaa.wasm"
MAX_BYTES=1572864
GZ_SIZE=$(ic-wasm "$AAA_WASM" -o /tmp/sc-aaa-shrunk.wasm shrink >/dev/null 2>&1 && gzip -c /tmp/sc-aaa-shrunk.wasm | wc -c | tr -d ' ')
[ "$GZ_SIZE" -le "$MAX_BYTES" ] || fail "aaa wasm ${GZ_SIZE} B gz exceeds 1.5 MiB"
ok "aaa wasm ${GZ_SIZE} B gz (≤ 1.5 MiB)"

OUT=$(POCKET_IC_MUTE_SERVER=1 cargo test --workspace -- --test-threads=4 2>&1)
STATUS=$?
if [ $STATUS -ne 0 ]; then echo "$OUT" | grep -E "FAILED|panicked|error" | head -30; fail "cargo test"; fi
PASSED=$(echo "$OUT" | grep -oE '[0-9]+ passed' | awk '{s+=$1} END {print s+0}')
IGNORED=$(echo "$OUT" | grep -oE '[0-9]+ ignored' | awk '{s+=$1} END {print s+0}')
[ "$IGNORED" -eq 0 ] || fail "$IGNORED ignored tests — a skipped test is a failed test"
echo "$OUT" | grep -qiE 'skipping|skipped' && fail "a test reported skipping — a skipped test is a failed test"
ok "cargo test: $PASSED passed, 0 ignored"

python3 scripts/coverage.py || fail "coverage below threshold"
ok "coverage gates"

if grep -rnE '\bdfx\b|fetchRootKey' crates frontend/ agent-kit tools --exclude-dir=node_modules --exclude-dir=.venv --exclude-dir=dist --exclude-dir=target --include='*.rs' --include='*.ts' --include='*.tsx' --include='*.py' 2>/dev/null | grep -q .; then
  fail "found dfx or fetchRootKey in source"
fi
ok "no dfx / fetchRootKey in source"

if [ -f tools/curation/pyproject.toml ] || [ -d tools/curation/tests ]; then
  just py-test >/dev/null 2>&1 || fail "pytest (tools/curation)"
  ok "pytest (tools/curation)"
fi

if [ -f frontend/package.json ]; then
  (cd frontend && npm run -s typecheck && npm run -s lint && npm run -s test && npm run -s build) >/tmp/sc-frontend.log 2>&1 || { tail -30 /tmp/sc-frontend.log; false; } || fail "frontend checks"
  ok "frontend typecheck, lint, test, build"
fi

if [ -f scripts/traceability.py ]; then
  python3 scripts/traceability.py || fail "traceability"
  ok "traceability"
fi

echo -e "${GREEN}All local gates passed.${NC}"
