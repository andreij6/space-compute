# Space Compute — Developer Task Runner (Justfile)
# Deterministic local development, testing, and deployment.

default:
    @just --list

# Deterministic one-shot local deploy (starts network, installs ledgers once, deploys canisters)
deploy-local:
    @bash scripts/deploy-local.sh

# Fast local verification suite (fmt, clippy, unit/integration tests, wasm size)
verify:
    @bash scripts/verify-local.sh

# Run all unit and integration tests (PocketIC)
test:
    @cargo test --workspace

# Run PocketIC integration tests only
test-integration:
    @cargo test -p integration-tests -- --nocapture

# Run acceptance demo for a specific task (e.g. just demo T1.1)
demo TASK_ID:
    @echo "Running demo for {{TASK_ID}}..."
    @cargo test --workspace -- "$(echo {{TASK_ID}} | tr 'A-Z.-' 'a-z__')_" --nocapture

# Regenerate docs/specs/10-tasks.md, the Gantt workbook and tasks.json from tools/plan/plan.py
plan:
    @test -x .venv/bin/python || (python3 -m venv .venv && .venv/bin/pip install -q -r requirements.txt)
    @.venv/bin/python tools/plan/plan.py

# Format all Rust code
fmt:
    @cargo fmt --all

# Run Clippy checks
clippy:
    @cargo clippy --all-targets -- -D warnings

# Clean build artifacts
clean:
    @cargo clean
    @rm -rf .env.local
