import csv
import json
import os
import urllib.request
from pathlib import Path

import numpy as np
from astropy import units as u
from astropy.coordinates import SkyCoord

GZC_URL = "https://zooniverse-data.s3.amazonaws.com/galaxy-zoo-candels/gz_candels_table_2_main_release.csv"
GZC_CACHE = Path(os.environ.get("SC_GZ_CACHE", Path.home() / ".cache" / "space-compute" / "gz" / "gz_candels_table_2_main_release.csv"))
GZC_FIELD = {"COS": "primer-cosmos", "GDS": "jades-gds", "UDS": "primer-uds"}
MIN_FRAC, MIN_VOTES, MATCH_ARCSEC, MAX_Z = 0.8, 20, 0.3, 2.0
STARS_PER_FIELD = 150
WEAK_PER_FIELD = 250

QUESTIONS = {
    "shape": ("t00_smooth_or_featured", {"smooth": "smooth", "features": "featured", "artifact": "artifact"}),
    "edgeon": ("t09_disk_edge_on", {"yes": "yes", "no": "no"}),
    "bar": ("t11_bar_feature", {"no": "none"}),
    "spiral": ("t12_spiral_pattern", {"yes": "yes", "no": "no"}),
    "clumps": ("t02_clumpy_appearance", {"no": "none"}),
    "merger": ("t16_merging_tidal_debris", {"neither": "none", "merging": "major"}),
}


def gz_answers(row: dict) -> dict:
    answers = {}
    for question, (task, mapping) in QUESTIONS.items():
        if float(row.get(f"{task}_count") or 0) < MIN_VOTES:
            continue
        for src, ours in mapping.items():
            key = next((k for k in row if k.startswith(task + "_a") and k.endswith(f"_{src}_frac") and "weighted" not in k), None)
            if key and float(row[key] or 0) >= MIN_FRAC:
                answers[question] = ours
    return answers


def strength(answers: dict) -> str:
    return "weak" if answers == {"merger": "none"} else "strong"


def load_gzc() -> list[dict]:
    if not GZC_CACHE.exists():
        GZC_CACHE.parent.mkdir(parents=True, exist_ok=True)
        urllib.request.urlretrieve(GZC_URL, GZC_CACHE)
    return list(csv.DictReader(open(GZC_CACHE)))


def match(gz_rows: list[dict], table, eligible: np.ndarray) -> dict:
    if not gz_rows or not eligible.any():
        return {}
    cat = SkyCoord(np.asarray(table["ra"])[eligible] * u.deg, np.asarray(table["dec"])[eligible] * u.deg)
    ids = np.asarray(table["subject_id"])[eligible]
    zs = np.asarray(table["z_phot"])[eligible]
    src = SkyCoord([float(r["RA"]) for r in gz_rows] * u.deg, [float(r["Dec"]) for r in gz_rows] * u.deg)
    idx, sep, _ = src.match_to_catalog_sky(cat)
    out = {}
    for row, i, s in zip(gz_rows, idx, sep.arcsec):
        if s > MATCH_ARCSEC or not (0 < zs[i] < MAX_Z):
            continue
        answers = gz_answers(row)
        if answers:
            out[int(ids[i])] = {"answers": answers, "strength": strength(answers),
                                "source": f"gz-candels:{row['ID']}", "sep_arcsec": round(float(s), 3)}
    return out


def stars(table, flags: dict) -> dict:
    rows = np.flatnonzero(flags["star"])
    order = rows[np.argsort(np.asarray(table["mag_f444w"])[rows])][:STARS_PER_FIELD]
    return {int(table["subject_id"][i]): {"answers": {"shape": "artifact"}, "strength": "strong", "source": "objective:point-source"} for i in order}


def run(out: Path, tables: dict, gz_rows: list[dict] | None = None) -> dict:
    from .select import classify

    gz_rows = gz_rows if gz_rows is not None else load_gzc()
    by_field = {}
    for r in gz_rows:
        f = GZC_FIELD.get(r["ID"][:3])
        if f:
            by_field.setdefault(f, []).append(r)
    gold, ids_by_field = {}, {}
    for field, table in tables.items():
        flags = classify(table)
        found = match(by_field.get(field, []), table, flags["eligible"])
        weak = sorted(k for k, v in found.items() if v["strength"] == "weak")
        rng = np.random.default_rng(len(field) * 7919)
        keep = set(rng.permutation(weak)[:WEAK_PER_FIELD].tolist()) if weak else set()
        found = {k: v for k, v in found.items() if v["strength"] == "strong" or k in keep}
        found.update(stars(table, flags))
        gold.update(found)
        ids_by_field[field] = sorted(found)
    summary = {"subjects": len(gold), "strong": sum(g["strength"] == "strong" for g in gold.values()),
               "weak": sum(g["strength"] == "weak" for g in gold.values()),
               "by_field": {f: len(v) for f, v in ids_by_field.items()}, "by_answer": {}}
    for g in gold.values():
        for q, a in g["answers"].items():
            key = f"{q}={a}"
            summary["by_answer"][key] = summary["by_answer"].get(key, 0) + 1
    out.mkdir(parents=True, exist_ok=True)
    (out / "gold_v1.json").write_text(json.dumps({str(k): v for k, v in sorted(gold.items())}, indent=1, sort_keys=True))
    return {"summary": summary, "ids_by_field": ids_by_field}
