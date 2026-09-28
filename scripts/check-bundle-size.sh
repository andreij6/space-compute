#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

DIST="src/frontend/dist"
INDEX="$DIST/index.html"
MAX_BYTES=358400

[ -f "$INDEX" ] || { echo "missing $INDEX; run npm run -s build in src/frontend first" >&2; exit 1; }

TOTAL=0
while IFS= read -r asset; do
  file="$DIST${asset}"
  [ -f "$file" ] || { echo "referenced asset missing on disk: $file" >&2; exit 1; }
  size=$(gzip -c "$file" | wc -c | tr -d ' ')
  TOTAL=$((TOTAL + size))
done < <(grep -oE '(src|href)="/assets/[^"]+\.js"' "$INDEX" | sed -E 's/^(src|href)="//; s/"$//')

echo "initial JS (index.html script + modulepreload): ${TOTAL} B gz (limit ${MAX_BYTES} B)"
[ "$TOTAL" -le "$MAX_BYTES" ] || { echo "initial JS bundle ${TOTAL} B gz exceeds 350 KiB" >&2; exit 1; }
