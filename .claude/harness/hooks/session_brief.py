#!/usr/bin/env python3
"""SessionStart: print a short brief (stdout is added to Claude's context)."""
import json, os, re
H = os.path.join(os.environ.get("CLAUDE_PROJECT_DIR", "."), ".claude/harness")
tasks = json.load(open(f"{H}/tasks.json"))
done = {t["id"] for t in tasks if t["status"] == "done"}
cur = open(f"{H}/current").read().strip() if os.path.exists(f"{H}/current") else ""
ready = [t for t in tasks if t["status"] == "todo" and all(d in done for d in t["deps"])][:4]
print(f"HARNESS — {len(done)}/{len(tasks)} tasks done. Protocol: .claude/harness/README.md (/task <id> then /retro).")
if cur: print(f"IN PROGRESS: {cur}")
for t in ready: print(f"READY {t['id']} [{t['tier']}→{t['model']}] {t['name']} · demo: {t['demo']}")
prog = open(f"{H}/progress.md").read().split("\n## ")
if len(prog) > 1: print("LAST: " + prog[1].split("\n")[0])
print("LESSONS:"); print("".join(l for l in open(f"{H}/LESSONS.md") if re.match(r"- L-\d+", l)).strip())
