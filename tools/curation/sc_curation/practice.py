import argparse
import json
from collections import defaultdict
from pathlib import Path

import numpy as np

from . import dossiers, gold, select

COUNT = 200


def labelled_pool(tables: dict, gz_rows: list[dict] | None = None) -> dict[int, dict]:
    gz_rows = gz_rows if gz_rows is not None else gold.load_gzc()
    by_field = {}
    for r in gz_rows:
        f = gold.GZC_FIELD.get(r["ID"][:3])
        if f:
            by_field.setdefault(f, []).append(r)
    pool = {}
    for field, table in tables.items():
        if field not in gold.GZC_FIELD.values():
            continue
        flags = select.classify(table)
        found = gold.match(by_field.get(field, []), table, flags["eligible"])
        for sid, rec in found.items():
            pool[sid] = {**rec, "field": field}
    return pool


def choose(pool: dict, selection_ids: set, gold_ids: set, count: int = COUNT) -> list[int]:
    exclude = set(selection_ids) | set(gold_ids)
    candidates = sorted(sid for sid in pool if sid not in exclude)
    if len(candidates) < count:
        raise ValueError(f"only {len(candidates)} non-overlapping labelled subjects available, need {count}")
    chosen = candidates[:count]
    assert set(chosen).isdisjoint(selection_ids), "practice/selection overlap"
    assert set(chosen).isdisjoint(gold_ids), "practice/gold overlap"
    return chosen


def build_rows(chosen: list[int], tables: dict) -> list[dict]:
    rows = []
    for field, table in tables.items():
        ids = np.asarray(table["subject_id"])
        idx = {int(i): k for k, i in enumerate(ids)}
        for sid in chosen:
            k = idx.get(sid)
            if k is None:
                continue
            r = table[k]
            rows.append({
                "subject_id": sid, "field": field, "root": str(r["root"]),
                "catalog_id": int(r["catalog_id"]), "ra_deg": round(float(r["ra"]), 7),
                "dec_deg": round(float(r["dec"]), 7),
            })
    rows.sort(key=lambda r: r["subject_id"])
    return rows


def select_practice(out: Path, tables: dict, gz_rows: list[dict] | None = None) -> dict:
    selection_ids = {json.loads(l)["subject_id"] for l in (out / "selection_v1.jsonl").read_text().splitlines()}
    gold_ids = {int(k) for k in json.loads((out / "gold_v1.json").read_text())}
    pool = labelled_pool(tables, gz_rows)
    chosen = choose(pool, selection_ids, gold_ids)
    rows = build_rows(chosen, tables)
    answers = sorted(({"subject_id": sid, "answers": pool[sid]["answers"]} for sid in chosen), key=lambda a: a["subject_id"])
    out.mkdir(parents=True, exist_ok=True)
    (out / "practice_v1.jsonl").write_text("".join(json.dumps(r, sort_keys=True) + "\n" for r in rows))
    (out / "practice_answers_v1.json").write_text(json.dumps(answers, indent=1, sort_keys=True))
    return {"rows": rows, "answers": answers}


def render_practice(rows: list[dict], answers: list[dict], out: Path, bucket: Path, work: Path, workers: int = 7) -> dict:
    by_root = defaultdict(list)
    for r in rows:
        by_root[r["root"]].append(r)
    manifest = []
    for root, root_rows in sorted(by_root.items()):
        workdir = work / root
        dossiers.cut_root(root, root_rows, workdir, workers)
        manifest += dossiers.assemble_root(root, root_rows, workdir, bucket)
    manifest.sort(key=lambda m: m["subject_id"])
    bucket.mkdir(parents=True, exist_ok=True)
    (bucket / "answers.json").write_text(json.dumps(answers, indent=1, sort_keys=True))
    (bucket / "manifest.jsonl").write_text("".join(json.dumps(m, sort_keys=True) + "\n" for m in manifest))
    out.mkdir(parents=True, exist_ok=True)
    (out / "practice_manifest_v1.jsonl").write_text("".join(json.dumps(m, sort_keys=True) + "\n" for m in manifest))
    return {"manifest": manifest}


def main() -> None:
    p = argparse.ArgumentParser(description="Build the practice_v1 set (200 GZ-labelled, non-task-pool dossiers)")
    p.add_argument("--out", default="../../data/curation/v1")
    p.add_argument("--bucket", default="../../target/bucket/practice_v1")
    p.add_argument("--work", default=str(select.CACHE.parent / "cutouts-practice"))
    p.add_argument("--workers", type=int, default=7)
    p.add_argument("--select-only", action="store_true")
    a = p.parse_args()
    out = Path(a.out)
    tables = {f: select.load_field(f) for f in gold.GZC_FIELD.values()}
    result = select_practice(out, tables)
    print(json.dumps({"count": len(result["rows"]), "fields": sorted({r["field"] for r in result["rows"]})}, indent=1))
    if not a.select_only:
        render_practice(result["rows"], result["answers"], out, Path(a.bucket), Path(a.work), a.workers)


if __name__ == "__main__":
    main()
