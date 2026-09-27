#!/usr/bin/env python3
import json, subprocess, sys
from collections import defaultdict

THRESHOLDS = {"platform": 90, "payments": 90, "treasury": 90, "aaa": 85, "sc-types": 85}
MEASURABLE_LINES = 50

out = subprocess.run(
    ["cargo", "llvm-cov", "--workspace", "--exclude", "integration-tests", "--exclude", "spike-probe",
     "--json", "--summary-only", "-q"],
    capture_output=True, text=True,
)
if out.returncode != 0:
    print(out.stderr[-2000:]); sys.exit(1)
files = json.loads(out.stdout)["data"][0]["files"]
lines = defaultdict(lambda: [0, 0])
for f in files:
    parts = f["filename"].split("/crates/")
    if len(parts) < 2 or "/src/" not in parts[1]:
        continue
    crate = parts[1].split("/")[0]
    s = f["summary"]["lines"]
    lines[crate][0] += s["covered"]
    lines[crate][1] += s["count"]

failed = False
print(f"{'crate':<10} {'lines':>6} {'covered':>8} {'min':>5}  status")
for crate, minimum in THRESHOLDS.items():
    cov, total = lines.get(crate, [0, 0])
    pct = 100.0 * cov / total if total else 100.0
    if total < MEASURABLE_LINES:
        status = f"not gated yet (< {MEASURABLE_LINES} lines)"
    elif pct + 1e-9 < minimum:
        status, failed = "FAIL", True
    else:
        status = "ok"
    print(f"{crate:<10} {total:>6} {pct:>7.1f}% {minimum:>4}%  {status}")
sys.exit(1 if failed else 0)
