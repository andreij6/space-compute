#!/usr/bin/env python3
import csv, os, sys, urllib.request
from collections import Counter

URL = "https://zooniverse-data.s3.amazonaws.com/galaxy-zoo-candels/gz_candels_table_2_main_release.csv"
CACHE = os.path.expanduser("~/.cache/space-compute/gz/gz_candels_table_2_main_release.csv")
FIELD = {"COS": "primer-cosmos", "GDS": "jades-gds", "UDS": "primer-uds"}
MIN_FRAC, MIN_VOTES = 0.8, 20

QUESTIONS = {
    "shape": ("t00_smooth_or_featured", {"smooth": "smooth", "features": "featured", "artifact": "artifact"}),
    "edgeon": ("t09_disk_edge_on", {"yes": "yes", "no": "no"}),
    "bar": ("t11_bar_feature", {"no": "none"}),
    "spiral": ("t12_spiral_pattern", {"yes": "yes", "no": "no"}),
    "clumps": ("t02_clumpy_appearance", {"no": "none"}),
    "merger": ("t16_merging_tidal_debris", {"neither": "none", "merging": "major"}),
}

if not os.path.exists(CACHE):
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    urllib.request.urlretrieve(URL, CACHE)

rows = list(csv.DictReader(open(CACHE)))
per_q, per_field, gold_subjects = Counter(), Counter(), 0
for r in rows:
    field = FIELD.get(r["ID"][:3])
    hit = False
    for q, (task, answers) in QUESTIONS.items():
        n = float(r.get(f"{task}_count") or 0)
        if n < MIN_VOTES:
            continue
        for src, ours in answers.items():
            key = next((k for k in r if k.startswith(task + "_a") and k.endswith(f"_{src}_frac") and "weighted" not in k), None)
            if key and float(r[key] or 0) >= MIN_FRAC:
                per_q[f"{q}={ours}"] += 1
                hit = True
    if hit:
        gold_subjects += 1
        per_field[field] += 1

print(f"GZ CANDELS rows: {len(rows)} (fields: COSMOS, GOODS-S, UDS)")
print(f"subjects with ≥1 gold-grade answer (top answer ≥{MIN_FRAC:.0%}, ≥{MIN_VOTES} votes): {gold_subjects}")
for f, n in sorted(per_field.items()):
    print(f"  {f}: {n}")
for k, n in sorted(per_q.items()):
    print(f"  {k}: {n}")
sys.exit(0 if gold_subjects >= 2000 else 1)
