import json
import random
from pathlib import Path

SEED = 20260928

CLAIMS = {
    "spiral": {
        "category": "other",
        "gold": ("spiral", {"yes"}),
        "quota": (3, 3),
        "wordings": [
            "Spiral arms wind out from a central bulge; worth recording the spiral structure.",
            "I can trace at least two arms curling around the core, so this reads as a spiral.",
            "Looks like a disk galaxy with well-defined spiral arms.",
        ],
    },
    "merger": {
        "category": "merger_interaction",
        "gold": ("merger", {"major"}),
        "quota": (9, 12),
        "wordings": [
            "Two bright cores with distorted, asymmetric light between them; looks like an ongoing major merger.",
            "The light profile is disturbed and there is a close companion of similar brightness, suggesting a merger.",
            "Double nucleus with bridging emission; I read this as two galaxies interacting.",
        ],
    },
    "edgeon": {
        "category": "other",
        "gold": ("edgeon", {"yes"}),
        "quota": (15, 19),
        "wordings": [
            "Thin, elongated disk seen almost exactly edge-on.",
            "The galaxy is a narrow streak with a slight central bulge, i.e. an edge-on disk.",
            "Very high axis ratio; this looks like a disk viewed side-on.",
        ],
    },
    "artifact": {
        "category": "artifact",
        "gold": ("shape", {"artifact"}),
        "quota": (25, 34),
        "wordings": [
            "Unresolved point-like source with spikes; this looks like a star or imaging artifact rather than a galaxy.",
            "No extended structure at all, so I am flagging it as an artifact and not a real galaxy.",
            "The source looks like detector or PSF residue rather than an astrophysical object.",
        ],
    },
}


def load_manifest(path: Path) -> dict[int, dict]:
    out = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        out[row["subject_id"]] = row
    return out


def claim_of(spec: dict) -> str:
    return next(k for k, c in CLAIMS.items() if c["category"] == spec["category"] and spec["rationale"] in c["wordings"])


def stratified(ids: list[int], manifest: dict[int, dict], rng: random.Random) -> list[int]:
    by_field: dict[str, list[int]] = {}
    for sid in sorted(ids):
        by_field.setdefault(manifest.get(sid, {}).get("field", ""), []).append(sid)
    groups = [by_field[f] for f in sorted(by_field)]
    for g in groups:
        rng.shuffle(g)
    out = []
    while any(groups):
        for g in groups:
            if g:
                out.append(g.pop())
    return out


def pick(pool: list[int], n: int, used: set[int]) -> list[int]:
    chosen = [sid for sid in pool if sid not in used][:n]
    if len(chosen) < n:
        raise ValueError("not enough gold subjects to build the honeypot pool")
    used.update(chosen)
    return chosen


def build(gold: dict, manifest: dict[int, dict] | None = None) -> list[dict]:
    manifest = manifest or {}
    rng = random.Random(SEED)
    used: set[int] = set()
    specs = []
    for claim in CLAIMS.values():
        question, agree_answers = claim["gold"]
        answered = {int(k): v["answers"][question] for k, v in gold.items() if question in v["answers"]}
        agree = stratified([s for s, a in answered.items() if a in agree_answers], manifest, rng)
        disagree = stratified([s for s, a in answered.items() if a not in agree_answers], manifest, rng)
        n_agree, n_disagree = claim["quota"]
        wordings = claim["wordings"]
        for truth, ids in (("Agree", pick(agree, n_agree, used)), ("Disagree", pick(disagree, n_disagree, used))):
            for i, sid in enumerate(ids):
                specs.append({
                    "subject_id": sid,
                    "category": claim["category"],
                    "rationale": wordings[i % len(wordings)],
                    "truth": truth,
                })
    return sorted(specs, key=lambda s: s["subject_id"])


def main(gold_path: Path, manifest_path: Path, out_path: Path) -> list[dict]:
    specs = build(json.loads(gold_path.read_text()), load_manifest(manifest_path))
    out_path.write_text(json.dumps(specs, indent=1))
    return specs


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[3]
    main(
        root / "data/curation/v1/gold_v1.json",
        root / "data/curation/v1/manifest_v1.jsonl",
        root / "data/curation/v1/honeypots_v1.json",
    )
