#!/usr/bin/env python3
import json, sys
tid, status = sys.argv[1], sys.argv[2]
p = ".claude/harness/tasks.json"
tasks = json.load(open(p))
hit = [t for t in tasks if t["id"] == tid]
if not hit: sys.exit(f"unknown task {tid}")
hit[0]["status"] = status
hit[0]["pct"] = 1 if status == "done" else (0.5 if status == "in_progress" else hit[0]["pct"])
json.dump(tasks, open(p, "w"), indent=1, ensure_ascii=False)
print(f"{tid} -> {status}")
