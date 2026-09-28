# Space Compute — Developer Task Runner (Justfile)
# Deterministic local development, testing, and deployment.

default:
    @just --list

# Deterministic one-shot local deploy (starts network, installs ledgers once, deploys canisters)
deploy-local:
    @bash scripts/deploy-local.sh

# Seed the local network: protocol v1, 500 subjects (SEED_SUBJECTS), AAA wasm; serves target/bucket on :8765
seed-local:
    @bash tools/seed-local/seed.sh

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

# Copy the Gantt tab rows (A2:K) to the clipboard for pasting into the Google Sheet (00-INDEX has the link)
gantt-paste:
    @python3 scripts/gantt_csv.py

# Set a task's status in tasks.json (todo | in_progress | done) and regenerate the plan
task-status TASK_ID STATUS:
    @python3 scripts/task-status.py {{TASK_ID}} {{STATUS}}
    @just plan >/dev/null

# Regenerate docs/specs/10-tasks.md, the Gantt workbook and tasks.json from tools/plan/plan.py
plan:
    @test -x .venv/bin/python || (python3 -m venv .venv && .venv/bin/pip install -q -r requirements.txt)
    @.venv/bin/python tools/plan/plan.py

# Regenerate committed .did files from the canister code
candid:
    @bash scripts/check-candid.sh --write

# Per-crate line coverage gate (platform/payments/treasury ≥ 90%, aaa/sc-types ≥ 85%)
coverage:
    @python3 scripts/coverage.py

# Curation pipeline tests (tools/curation, uv-managed venv)
py-test:
    @cd tools/curation && uv run -q pytest -q

# Generate the 50 synthetic fixture dossiers into target/fixtures
fixtures:
    @cd tools/curation && uv run -q python -m sc_curation.fixtures --out ../../target/fixtures

# Download DJA v7 catalogues (~4 GB, cached in ~/.cache/space-compute/dja)
curate-fetch:
    @cd tools/curation && uv run -q python -m sc_curation.fetch

# Select 20k subjects + gold into data/curation/v1
curate-select:
    @cd tools/curation && uv run -q python -m sc_curation.select --out ../../data/curation/v1 --total 5000

# Stream mosaics and render every dossier into target/bucket (resumable per root)
curate-dossiers *ARGS:
    @cd tools/curation && uv run -q python -m sc_curation.dossiers {{ARGS}}

# Automated + visual QA of 40 random dossiers
curate-qa:
    @cd tools/curation && uv run -q python -m sc_curation.qa

# Build the 200-subject practice_v1 set (GZ-labelled, excluded from selection/gold) with answers
curate-practice:
    @cd tools/curation && uv run -q python -m sc_curation.practice --out ../../data/curation/v1 --bucket ../../target/bucket/practice_v1

# Export resolved discoveries + citations from the local network, then build the v0 open-data release
release-data:
    @cd tools/curation && uv run -q python -m sc_curation.export_public --out ../../target/curation/v1/discoveries_export.json
    @cd tools/curation && uv run -q python -m sc_curation.release --discoveries ../../target/curation/v1/discoveries_export.json --out ../../target/release/v0

# Verify every hash in target/bucket, then upload to R2 (needs R2_* env vars; owner task T8.14)
publish-data:
    @cd tools/curation && uv run -q python -m sc_curation.publish --upload

# Fuzz sc-types decoders and validators (nightly toolchain; default 60 s per target)
fuzz SECS='60':
    @bash scripts/fuzz.sh {{SECS}}

# Nightly, non-blocking suite: 10 min fuzzing per target (LLM agent run and load test join later)
nightly:
    @bash scripts/fuzz.sh 600

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
