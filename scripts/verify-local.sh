#!/usr/bin/env bash
# =============================================================================
# verify-local.sh — Deterministic local test & verification suite
#
# Replaces remote CI with fast, reproducible local validation.
# Run before committing or marking any task done.
# =============================================================================

set -euo pipefail
cd "$(dirname "$0")/.."

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; BLUE='\033[0;34m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✅${NC} $1"; }
note() { echo -e "${YELLOW}ℹ️ ${NC} $1"; }
fail() { echo -e "${RED}❌${NC} $1"; exit 1; }

echo -e "${BLUE}🔭 Space Compute — Local Verification Suite${NC}"
echo "──────────────────────────────────────────────────"

# 1. Format check
if [ -f "Cargo.toml" ]; then
  note "Checking Rust formatting (cargo fmt --check)..."
  cargo fmt --all -- --check || fail "Rust code formatting check failed. Run 'cargo fmt --all' to fix."
  ok "Formatting clean"
fi

# 2. Clippy linter
if [ -f "Cargo.toml" ]; then
  note "Running Clippy (cargo clippy --all-targets -- -D warnings)..."
  cargo clippy --all-targets -- -D warnings || fail "Clippy warnings detected."
  ok "Clippy passed without warnings"
fi

# 3. Unit & Integration Tests
if [ -f "Cargo.toml" ]; then
  note "Running cargo tests (native + integration)..."
  # Disallow test skips from masking failures (L-007)
  TEST_OUTPUT=$(cargo test --workspace -- --nocapture 2>&1) || {
    echo "$TEST_OUTPUT"
    fail "Cargo tests failed."
  }
  echo "$TEST_OUTPUT" | grep -E "test result:" || true
  if echo "$TEST_OUTPUT" | grep -q "0 passed; 0 failed"; then
    note "No cargo tests defined yet."
  else
    ok "All unit & integration tests passed"
  fi
fi

# 4. AAA Wasm Size Budget Check (<= 1.5 MiB gz)
AAA_WASM="target/wasm32-unknown-unknown/release/aaa.wasm"
if [ -f "$AAA_WASM" ]; then
  note "Checking AAA wasm size budget (<= 1.5 MiB gz)..."
  GZ_SIZE=$(gzip -c "$AAA_WASM" | wc -c | tr -d ' ')
  MAX_BYTES=$(( 15 * 1024 * 1024 / 10 )) # 1.5 MiB = 1,572,864 bytes
  if [ "$GZ_SIZE" -gt "$MAX_BYTES" ]; then
    fail "AAA wasm size ($GZ_SIZE bytes gz) exceeds 1.5 MiB budget ($MAX_BYTES bytes)."
  fi
  ok "AAA wasm size budget compliant ($GZ_SIZE / $MAX_BYTES bytes gz)"
fi

echo "──────────────────────────────────────────────────"
ok "Local verification suite completed successfully."
