#!/usr/bin/env python3
import json, re, subprocess, sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]


def section(spec, heading_pat):
    text = (root / f"docs/specs/{spec}").read_text()
    m = re.search(heading_pat + r'[^\n]*\n', text)
    if not m:
        return ""
    rest = text[m.end():]
    m2 = re.search(r'\n## ', rest)
    return rest[:m2.start()] if m2 else rest


def numbered_items(body, start=1):
    items = {}
    for m in re.finditer(r'^(\d+)\.\s+(.*)$', body, re.M):
        items[int(m.group(1))] = m.group(2).strip()
    return items


def bullet_items(body):
    return [m.group(1).strip() for m in re.finditer(r'^-\s+(.*)$', body, re.M)]


def threat_codes(body):
    return {m.group(1): m.group(2).strip() for m in re.finditer(r'^\|\s*(S\d+)\s*\|\s*(.*?)\s*\|', body, re.M)}


required = {}

for n, txt in numbered_items(section("02-platform-canister.md", r'## 11\. Acceptance criteria')).items():
    required[f"02§11#{n}"] = txt
for n, txt in numbered_items(section("03-aaa-canister.md", r'## 8\. Acceptance criteria.*')).items():
    required[f"03§8#{n}"] = txt
for n, txt in numbered_items(section("04-payments-canister.md", r'## 5\. Acceptance criteria.*')).items():
    required[f"04§5#{n}"] = txt
for n, txt in numbered_items(section("05-frontend.md", r'## 5\. Acceptance')).items():
    required[f"05§5#{n}"] = txt
for n, txt in numbered_items(section("06-agent-toolkit.md", r'## 5\. Acceptance')).items():
    required[f"06§5#{n}"] = txt
for i, txt in enumerate(bullet_items(section("07-data-curation.md", r'## 7\. Acceptance')), start=1):
    required[f"07§7#{i}"] = txt
for code, txt in threat_codes(section("08-security.md", r'## 2\. Threats.*')).items():
    required[f"08§2#{code}"] = txt
required["08§3"] = "Input limits enforced in canisters"
required["08§4"] = "Payments review gate signed"
required["08§7"] = "Manual upgrade path for a self-managed AAA"
required["09§1#upgrade"] = "vN-1 -> vN state survives, per canister"
required["09§1#property"] = "proptest invariants: evaluate() monotone, replay==incremental, protocol paths"
for n, txt in numbered_items(section("12-treasury-keeper.md", r'## 5\. Acceptance.*')).items():
    required[f"12§5#{n}"] = txt

TAG_RE = re.compile(r'(\d{2}[a-z]?)\s*§\s*(\d+[a-z]?)(?:\.(\d+)|\s*#\s*([\d,\-]+))?')
S_RE = re.compile(r'\bS(\d{1,2})\b')


def row_tags(item_text):
    tags = set()
    for spec, sec, dotitem, hashitems in TAG_RE.findall(item_text):
        base = f"{spec}§{sec}"
        if dotitem:
            tags.add(f"{base}#{dotitem}")
        elif hashitems:
            for part in hashitems.split(','):
                part = part.strip()
                if '-' in part:
                    a, b = part.split('-')
                    tags.update(f"{base}#{i}" for i in range(int(a), int(b) + 1))
                elif part:
                    tags.add(f"{base}#{part}")
        else:
            tags.add(base)
    for n in S_RE.findall(item_text):
        tags.add(f"08§2#S{n}")
    low = item_text.lower()
    if re.search(r'09\s*§\s*1', item_text):
        if "upgrade" in low:
            tags.add("09§1#upgrade")
        if "proptest" in low or "property" in low:
            tags.add("09§1#property")
    return tags


text = (root / "docs/specs/traceability.md").read_text()
main, _, waiver_text = text.partition("## Waivers")

rows = []
for line in main.splitlines():
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) == 3 and cells[1] and not cells[1].startswith(("Task", "-")):
        rows.append({"item": cells[0], "task": cells[1], "proof": cells[2], "tags": row_tags(cells[0])})

waivers = {}
for line in waiver_text.splitlines():
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) == 2 and cells[0] and not cells[0].startswith(("Task", "Item", "-")):
        waivers[cells[0]] = cells[1]

sources = subprocess.run(
    ["grep", "-rhoE", r"(fn|def) (t[0-9]+[a-z]*_[0-9]+_[a-z0-9_]+|test_t[0-9]+[a-z]*_[0-9]+_[a-z0-9_]+|sp_[0-9]+_[a-z0-9_]+)",
     "crates", "tools", "agent-kit", "fuzz", "src/frontend", "--include=*.rs", "--include=*.py", "--include=*.ts", "--include=*.tsx"],
    cwd=root, capture_output=True, text=True).stdout
defined = {line.split()[1] for line in sources.splitlines()}


CANISTER_SRC = ["crates/platform/src", "crates/payments/src", "crates/treasury/src", "crates/aaa/src"]
CHECKS = {
    "bounded-wait-only": {
        "forbid": r"unbounded_wait|ic_cdk::call\(|ic_cdk::api::call::|call_with_payment",
        "require": r"Call::bounded_wait",
        "paths": CANISTER_SRC,
    },
}
DOC_PROOF_RE = re.compile(r"^(docs|okf)/|\.(md|png|jpe?g|gif|txt|csv)$")
DOC_TAG = "[doc-proof]"


def grep(pattern, paths):
    return subprocess.run(["grep", "-rnE", pattern, *paths], cwd=root, capture_output=True, text=True).stdout


def check_ok(name):
    c = CHECKS.get(name)
    if not c or not all((root / p).exists() for p in c["paths"]):
        return False
    return not grep(c["forbid"], c["paths"]) and bool(grep(c["require"], c["paths"]))


def proof_ok(proof):
    if proof.startswith("check:"):
        return check_ok(proof[len("check:"):])
    if "/" in proof:
        return (root / proof).exists()
    return proof in defined


def is_doc_proof(r):
    return bool(DOC_PROOF_RE.search(r["proof"]))


dangling = [r for r in rows if not proof_ok(r["proof"])]
doc_flagged = [r for r in rows if is_doc_proof(r) and DOC_TAG not in r["item"]]


def covers(r):
    return proof_ok(r["proof"]) and r not in doc_flagged

unmapped = []
mapped = []
for tag, txt in required.items():
    covering = [r for r in rows if tag in r["tags"] and covers(r)]
    if covering:
        mapped.append((tag, txt, covering))
    elif tag in waivers:
        pass
    else:
        unmapped.append((tag, txt))

tasks = json.loads((root / ".claude/harness/tasks.json").read_text())
covered_tasks = {r["task"] for r in rows}
task_unmapped = []
for t in tasks:
    if t["status"] == "done" and t["tier"] != "-" and t["id"] not in covered_tasks:
        if t["id"] not in waivers:
            task_unmapped.append(t["id"])

lines = ["# Traceability audit", "", f"Generated by `scripts/traceability_audit.py`. {len(required)} spec acceptance items, {len(rows)} traceability rows, {len(tasks)} tasks.", ""]
lines += ["## Unmapped acceptance items", ""]
if unmapped:
    for tag, txt in unmapped:
        lines.append(f"- `{tag}`: {txt}")
else:
    lines.append("None.")
lines += ["", "## Dangling traceability rows (proof missing)", ""]
if dangling:
    for r in dangling:
        lines.append(f"- {r['task']}: `{r['proof']}` — {r['item']}")
else:
    lines.append("None.")
lines += ["", f"## Doc-path proofs not tagged `{DOC_TAG}` (need a test, or the tag if a document is the proof)", ""]
if doc_flagged:
    for r in doc_flagged:
        lines.append(f"- {r['task']}: `{r['proof']}` — {r['item']}")
else:
    lines.append("None.")
lines += ["", "## Done tasks with no traceability row", ""]
if task_unmapped:
    for tid in task_unmapped:
        lines.append(f"- {tid}")
else:
    lines.append("None.")
lines += ["", "## Waived", ""]
if waivers:
    for tid, reason in waivers.items():
        lines.append(f"- `{tid}`: {reason}")
else:
    lines.append("None.")
lines += ["", f"## Mapped ({len(mapped)}/{len(required)})", ""]
for tag, txt, covering in sorted(mapped):
    lines.append(f"- `{tag}` -> {', '.join(sorted({c['proof'] for c in covering}))}")
(root / "docs/specs/traceability-audit.md").write_text("\n".join(lines) + "\n")

errors = len(unmapped) + len(dangling) + len(task_unmapped) + len(doc_flagged)
print(f"traceability_audit: {len(mapped)}/{len(required)} items mapped, {len(unmapped)} unmapped, "
      f"{len(dangling)} dangling rows, {len(doc_flagged)} untagged doc proofs, {len(task_unmapped)} tasks without a row, "
      f"{len(waivers)} waived")
for tag, txt in unmapped:
    print(f"traceability_audit: UNMAPPED {tag}: {txt}")
for r in dangling:
    print(f"traceability_audit: DANGLING {r['task']}: proof {r['proof']} not found")
for r in doc_flagged:
    print(f"traceability_audit: DOC PROOF {r['task']}: {r['proof']} (tag the item {DOC_TAG} or prove it with a test)")
for tid in task_unmapped:
    print(f"traceability_audit: NO ROW {tid}")
sys.exit(1 if errors else 0)
