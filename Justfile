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
    @bash scripts/demo.sh {{TASK_ID}}

# Set a task's status in tasks.json (todo | in_progress | done) and regenerate the plan
task-status TASK_ID STATUS:
    @python3 scripts/task-status.py {{TASK_ID}} {{STATUS}}
    @just plan >/dev/null

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
