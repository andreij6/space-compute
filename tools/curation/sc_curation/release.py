import argparse
import csv
import hashlib
import io
import json
from pathlib import Path

LICENSE = "CC BY 4.0"
CSV_COLUMNS = [
    "public_id", "subject_id", "category", "status", "confidence",
    "discoverer_name", "resolved_at", "text", "rationale",
]


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_jsonl(path: Path) -> list[dict]:
    return [json.loads(l) for l in path.read_text().splitlines()] if path.exists() else []


def hidden_subjects(curation_dir: Path) -> set[int]:
    gold = curation_dir / "gold_v1.json"
    honeypots = curation_dir / "honeypots_v1.json"
    ids = {int(k) for k in json.loads(gold.read_text())} if gold.exists() else set()
    return ids | ({h["subject_id"] for h in json.loads(honeypots.read_text())} if honeypots.exists() else set())


def consensus_row(row: dict) -> dict:
    out = {"subject_id": row["subject_id"], "resolved_at": row["resolved_at"], "consensus": dict(row["consensus"])}
    if row.get("votes"):
        out["vote_fractions"] = {
            q: {a: round(n / sum(counts.values()), 4) for a, n in counts.items()}
            for q, counts in row["votes"].items()
        }
    return out


def readme(n_discoveries: int, n_subjects: int, n_consensus: int) -> str:
    return (
        "# Space Compute open data release v0\n\n"
        f"{n_discoveries} resolved discoveries with citations and consensus classifications for "
        f"{n_consensus} retired subjects, drawn from {n_subjects} published subjects.\n\n"
        f"License: {LICENSE}. Built deterministically from the platform's public queries "
        "(`list_discoveries` / `get_citation` / `get_subject_consensus`) and the curation pipeline's manifest/selection metadata.\n\n"
        "Files:\n"
        "- `discoveries.json` / `discoveries.csv` -- resolved discoveries with citation text and credits\n"
        "- `manifest.jsonl` -- published subject dossiers (id, field, coords, image/dossier hashes)\n"
        "- `selection.jsonl` -- the v1 task-pool selection (catalogue id, coords, photometry)\n"
        "- `consensus.jsonl` -- per-subject consensus answers and vote fractions from resolved classifications\n"
        "- `CHECKSUMS.json` -- sha256 of every file in this release plus the combined release hash\n"
    )


def discoveries_csv(rows: list[dict]) -> str:
    buf = io.StringIO()
    w = csv.DictWriter(buf, fieldnames=CSV_COLUMNS, extrasaction="ignore")
    w.writeheader()
    for row in rows:
        flat = {**row.get("discovery", {}), **(row.get("citation") or {})}
        w.writerow({k: flat.get(k, "") for k in CSV_COLUMNS})
    return buf.getvalue()


def build(discoveries: list[dict], curation_dir: Path, out_dir: Path, consensus: list[dict] = ()) -> dict:
    discoveries = sorted(discoveries, key=lambda r: r["discovery"]["public_id"])
    manifest = sorted(load_jsonl(curation_dir / "manifest_v1.jsonl"), key=lambda m: m["subject_id"])
    selection = sorted(
        ({k: v for k, v in r.items() if k != "reason"} for r in load_jsonl(curation_dir / "selection_v1.jsonl")),
        key=lambda r: r["subject_id"],
    )
    hidden = hidden_subjects(curation_dir)
    consensus = sorted(
        (consensus_row(c) for c in consensus if c["subject_id"] not in hidden),
        key=lambda c: c["subject_id"],
    )

    files = {
        "README.md": readme(len(discoveries), len(manifest), len(consensus)),
        "discoveries.json": json.dumps(discoveries, indent=1, sort_keys=True),
        "discoveries.csv": discoveries_csv(discoveries),
        "manifest.jsonl": "".join(json.dumps(m, sort_keys=True) + "\n" for m in manifest),
        "selection.jsonl": "".join(json.dumps(r, sort_keys=True) + "\n" for r in selection),
        "consensus.jsonl": "".join(json.dumps(c, sort_keys=True) + "\n" for c in consensus),
    }
    out_dir.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for name in sorted(files):
        body = files[name].encode()
        (out_dir / name).write_bytes(body)
        hashes[name] = sha256_bytes(body)
    release_sha256 = sha256_bytes(json.dumps(hashes, sort_keys=True).encode())
    (out_dir / "CHECKSUMS.json").write_text(json.dumps({"files": hashes, "release_sha256": release_sha256}, indent=1, sort_keys=True))
    return {"files": hashes, "release_sha256": release_sha256, "discoveries": len(discoveries), "subjects": len(manifest)}


def main() -> None:
    p = argparse.ArgumentParser(description="Build the v0 open-data release from an exported discoveries JSON")
    p.add_argument("--discoveries", default="../../target/curation/v1/discoveries_export.json")
    p.add_argument("--consensus", default="../../target/curation/v1/consensus_export.json")
    p.add_argument("--curation-dir", default="../../data/curation/v1")
    p.add_argument("--out", default="../../target/release/v0")
    a = p.parse_args()
    path = Path(a.discoveries)
    discoveries = json.loads(path.read_text()) if path.exists() else []
    consensus_path = Path(a.consensus)
    consensus = json.loads(consensus_path.read_text()) if consensus_path.exists() else []
    result = build(discoveries, Path(a.curation_dir), Path(a.out), consensus)
    print(json.dumps(result, indent=1))


if __name__ == "__main__":
    main()
