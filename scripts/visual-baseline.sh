#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../src/frontend"
if [ "${1:-}" = "--update" ]; then
  npx playwright test tests/e2e/visual.spec.ts --update-snapshots
else
  npx playwright test tests/e2e/visual.spec.ts
fi
