#!/usr/bin/env python3
import json, re, subprocess, sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
rows, waivers = [], {}
text = (root / "docs/specs/traceability.md").read_text()
main, _, waiver_text = text.partition("## Waivers")
for line in main.splitlines():
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) == 3 and cells[1] and not cells[1].startswith(("Task", "-")):
        rows.append(cells)
for line in waiver_text.splitlines():
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) == 2 and cells[0] and not cells[0].startswith(("Task", "-")):
        waivers[cells[0]] = cells[1]

sources = subprocess.run(
    ["grep", "-rhoE", r"(fn|def) (t[0-9]+_[0-9]+_[a-z0-9_]+|test_t[0-9]+_[0-9]+_[a-z0-9_]+|sp_[0-9]+_[a-z0-9_]+)",
     "crates", "tools", "agent-kit", "fuzz", "--include=*.rs", "--include=*.py"],
    cwd=root, capture_output=True, text=True).stdout
defined = {line.split()[1] for line in sources.splitlines()}

errors = []
for item, task, proof in rows:
    if "/" in proof:
        if not (root / proof).exists():
            errors.append(f"{task}: proof script {proof} missing")
    elif proof not in defined:
        errors.append(f"{task}: test {proof} not found")

tasks = json.loads((root / ".claude/harness/tasks.json").read_text())
covered = {t for _, t, _ in rows}
for t in tasks:
    if t["status"] == "done" and t["tier"] != "-" and t["id"] not in covered:
        if t["id"] in waivers:
            print(f"traceability: WAIVED {t['id']}: {waivers[t['id']]}")
            continue
        errors.append(f"{t['id']} is done but has no traceability row")

for e in errors:
    print(f"traceability: {e}")
print(f"traceability: {len(rows)} rows, {len(errors)} problems")
sys.exit(1 if errors else 0)
