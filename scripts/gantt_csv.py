#!/usr/bin/env python3
import json, subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[1]
tasks = json.loads((root / ".claude/harness/tasks.json").read_text())
names = {"P0": "Planning", "P1": "Foundations & spikes", "P2": "Platform core", "P3": "AAA canister & agent kit",
         "P4": "Review, consensus & credits", "P5": "Payments", "P6": "Frontend", "P7": "Hardening",
         "P8": "Beta & launch", "P9": "Design & polish"}
rows = []
for i, t in enumerate(tasks, start=2):
    who = "you" if not t["lane"] else f"agent L{t['lane']} ({t['model']})"
    rows.append("\t".join([t["id"], f"{t['phase']} {names[t['phase']]}", t["name"], who, t["start"], t["end"], str(t["days"]),
                           ", ".join(t["deps"]), f"{round(t['pct'] * 100)}%",
                           f'=IF(I{i}>=1,"Done",IF(TODAY()>F{i},"Late",IF(I{i}>0,"In progress","Not started")))', f"=G{i}*I{i}"]))
subprocess.run(["pbcopy"], input="\n".join(rows).encode(), check=True)
print(f"{len(rows)} rows copied. In the Gantt sheet: select A2:K200, Delete, click A2, Cmd+V.")
