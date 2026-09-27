#!/usr/bin/env python3
import csv, json, sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
tasks = json.loads((root / ".claude/harness/tasks.json").read_text())
notes = json.loads((root / ".claude/harness/sheet_notes.json").read_text()) if (root / ".claude/harness/sheet_notes.json").exists() else {}
w = csv.writer(sys.stdout)
w.writerow(["ID", "Phase", "Task", "Model", "Lane", "Start", "End", "Days", "Status", "% Complete", "Proof / note"])
label = {"done": "Done", "in_progress": "In progress", "todo": "Not started"}
for t in tasks:
    w.writerow([t["id"], t["phase"], t["name"][:70], t["model"], t["lane"] or "you", t["start"], t["end"], t["days"],
                notes.get(t["id"], {}).get("status", label[t["status"]]), f"{round(t['pct'] * 100)}%",
                notes.get(t["id"], {}).get("note", f"just demo {t['id']}" if t["status"] == "done" and t["lane"] else "")])
