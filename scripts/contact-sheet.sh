#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../src/frontend"
mkdir -p ../../docs/demos/T9.6
CONTACT_SHEET=1 npx playwright test tests/e2e/contact-sheet.spec.ts
ls -1 ../../docs/demos/T9.6/contact-sheet-*.png
