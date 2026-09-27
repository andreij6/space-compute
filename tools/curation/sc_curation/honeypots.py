import json
from pathlib import Path

CATEGORIES = [
    "lensed_arc",
    "merger_interaction",
    "clumpy_disk",
    "little_red_dot",
    "high_z_candidate",
    "ring",
    "tidal_feature",
    "unusual_color",
    "artifact",
    "other",
]

FALSE_RATIONALE = {
    "lensed_arc": "Faint curved arc-like feature hugging the core; flagging as a possible gravitational lens.",
    "merger_interaction": "Asymmetric light and a close companion suggest an ongoing merger.",
    "clumpy_disk": "Several bright clumps scattered across the disk; reads as clumpy star formation.",
    "little_red_dot": "Compact core with a red F277W-F444W colour; candidate little red dot / high-z AGN.",
    "high_z_candidate": "Dropout in the bluer bands with a red colour; likely a high-z dropout.",
    "ring": "A faint ring of light encircling the nucleus.",
    "tidal_feature": "Low surface brightness tail extending from the main body; possible tidal debris.",
    "unusual_color": "Colour looks inconsistent with the expected photo-z for this field.",
    "artifact": "Diffraction-spike-like pattern crossing the source; could be an imaging artifact.",
    "other": "Something about the morphology looks unusual and worth a second look.",
}

TRUE_RATIONALE = {
    "merger_interaction": "Clear double nucleus and a tidal tail; a textbook major merger.",
}


def load_manifest(path: Path) -> dict[int, dict]:
    out = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        out[row["subject_id"]] = row
    return out


def cite(rationale: str, dossier: dict | None) -> str:
    if not dossier:
        return rationale
    return f"{rationale} ({dossier['field']} field, RA {dossier['ra_deg']:.4f}, Dec {dossier['dec_deg']:.4f})"


def truth_pool(gold: dict) -> list[tuple[int, str, str]]:
    return sorted(
        (int(sid), "merger_interaction", "Agree")
        for sid, row in gold.items()
        if row["answers"].get("merger") == "major"
    )


def false_pool(gold: dict, exclude: set[int]) -> list[int]:
    return sorted(int(sid) for sid in gold if int(sid) not in exclude)


def build(gold: dict, manifest: dict[int, dict] | None = None, count: int = 120) -> list[dict]:
    manifest = manifest or {}
    truths = truth_pool(gold)
    exclude = {sid for sid, _, _ in truths}
    clean = false_pool(gold, exclude)
    need_false = count - len(truths)
    if need_false < 0 or need_false > len(clean):
        raise ValueError("not enough gold subjects to build the honeypot pool")

    specs = [
        {
            "subject_id": sid,
            "category": cat,
            "rationale": cite(TRUE_RATIONALE[cat], manifest.get(sid)),
            "truth": truth,
        }
        for sid, cat, truth in truths
    ]
    for i in range(need_false):
        sid = clean[i]
        cat = CATEGORIES[i % len(CATEGORIES)]
        specs.append(
            {
                "subject_id": sid,
                "category": cat,
                "rationale": cite(FALSE_RATIONALE[cat], manifest.get(sid)),
                "truth": "Disagree",
            }
        )
    return sorted(specs, key=lambda s: s["subject_id"])


def main(gold_path: Path, manifest_path: Path, out_path: Path, count: int = 120) -> list[dict]:
    gold = json.loads(gold_path.read_text())
    manifest = load_manifest(manifest_path)
    specs = build(gold, manifest, count)
    out_path.write_text(json.dumps(specs, indent=1))
    return specs


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[3]
    main(
        root / "data/curation/v1/gold_v1.json",
        root / "data/curation/v1/manifest_v1.jsonl",
        root / "data/curation/v1/honeypots_v1.json",
    )
