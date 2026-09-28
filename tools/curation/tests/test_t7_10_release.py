import json
from pathlib import Path

from sc_curation import release

ROOT = Path(__file__).resolve().parents[3]
CURATION = ROOT / "data" / "curation" / "v1"

DISCOVERIES = [
    {
        "discovery": {"public_id": "sc-2", "subject_id": 20000102, "category": "clumpy_disk", "status": "Confirmed",
                      "confidence": 80, "resolved_at": 200, "discoverer_name": "bee"},
        "citation": {"public_id": "sc-2", "text": "A clumpy disk in JADES-GDS.", "rationale": "Several bright clumps.",
                     "category": "clumpy_disk", "outcome": "Confirmed", "resolved_at": 200},
    },
    {
        "discovery": {"public_id": "sc-1", "subject_id": 20000088, "category": "merger_interaction", "status": "Confirmed",
                      "confidence": 90, "resolved_at": 100, "discoverer_name": "ant"},
        "citation": {"public_id": "sc-1", "text": "A merger in JADES-GDS.", "rationale": "Double nucleus.",
                     "category": "merger_interaction", "outcome": "Confirmed", "resolved_at": 100},
    },
]


def test_t7_10_release_build_is_reproducible_byte_for_byte(tmp_path):
    a = release.build(DISCOVERIES, CURATION, tmp_path / "a")
    b = release.build(DISCOVERIES, CURATION, tmp_path / "b")
    assert a["release_sha256"] == b["release_sha256"]
    assert a["files"] == b["files"]
    for name in a["files"]:
        assert (tmp_path / "a" / name).read_bytes() == (tmp_path / "b" / name).read_bytes()


def test_t7_10_release_sha256_changes_when_content_changes(tmp_path):
    a = release.build(DISCOVERIES, CURATION, tmp_path / "a")
    changed = [dict(DISCOVERIES[0]), DISCOVERIES[1]]
    changed[0] = {**changed[0], "discovery": {**changed[0]["discovery"], "confidence": 1}}
    b = release.build(changed, CURATION, tmp_path / "b")
    assert a["release_sha256"] != b["release_sha256"]


def test_t7_10_release_discoveries_are_sorted_by_public_id(tmp_path):
    release.build(DISCOVERIES, CURATION, tmp_path / "out")
    rows = json.loads((tmp_path / "out" / "discoveries.json").read_text())
    assert [r["discovery"]["public_id"] for r in rows] == ["sc-1", "sc-2"]


GOLD = json.loads((CURATION / "gold_v1.json").read_text())
HONEYPOTS = json.loads((CURATION / "honeypots_v1.json").read_text())
GOLD_ID = int(next(iter(GOLD)))
HONEYPOT_ID = HONEYPOTS[0]["subject_id"]
CONSENSUS = [
    {"subject_id": 10000086, "consensus": [["shape", "smooth"], ["merger", "none"]], "resolved_at": 300,
     "votes": {"shape": {"smooth": 3, "featured": 1}, "merger": {"none": 4}}},
    {"subject_id": 10000330, "consensus": [["shape", "featured"]], "resolved_at": 400},
    {"subject_id": GOLD_ID, "consensus": [["shape", "artifact"]], "resolved_at": 500,
     "votes": {"shape": {"artifact": 5}}},
    {"subject_id": HONEYPOT_ID, "consensus": [["shape", "artifact"]], "resolved_at": 600},
]


def release_rows(out):
    for path in sorted(out.iterdir()):
        text = path.read_text()
        if path.suffix == ".jsonl":
            yield path.name, [json.loads(l) for l in text.splitlines()]
        elif path.suffix == ".json":
            yield path.name, json.loads(text)


def keys_of(node):
    if isinstance(node, dict):
        for k, v in node.items():
            yield k
            yield from keys_of(v)
    elif isinstance(node, list):
        for v in node:
            yield from keys_of(v)


def test_t7_10_release_includes_manifest_selection_and_consensus(tmp_path):
    release.build(DISCOVERIES, CURATION, tmp_path / "out", CONSENSUS)
    manifest = (tmp_path / "out" / "manifest.jsonl").read_text().splitlines()
    selection = (tmp_path / "out" / "selection.jsonl").read_text().splitlines()
    consensus = [json.loads(l) for l in (tmp_path / "out" / "consensus.jsonl").read_text().splitlines()]
    assert len(manifest) == 5000
    assert len(selection) == 5000
    assert [c["subject_id"] for c in consensus] == [10000086, 10000330]
    assert consensus[0]["consensus"] == {"merger": "none", "shape": "smooth"}
    assert consensus[0]["vote_fractions"] == {"merger": {"none": 1.0}, "shape": {"featured": 0.25, "smooth": 0.75}}
    assert "vote_fractions" not in consensus[1]


def test_t7_10_release_never_ships_gold_answers_or_honeypot_truth(tmp_path):
    out = tmp_path / "out"
    release.build(DISCOVERIES, CURATION, out, CONSENSUS)
    names = {p.name for p in out.iterdir()}
    assert not any("gold" in n or "honeypot" in n for n in names)
    for name, rows in release_rows(out):
        keys = set(keys_of(rows))
        assert not keys & {"answers", "gold", "truth", "honeypot_truth", "is_honeypot", "strength", "source", "reason"}, name
    consensus_ids = {json.loads(l)["subject_id"] for l in (out / "consensus.jsonl").read_text().splitlines()}
    assert not consensus_ids & ({int(k) for k in GOLD} | {h["subject_id"] for h in HONEYPOTS})
    blob = "\n".join(p.read_text() for p in out.iterdir())
    assert "gold" not in blob.lower()
    assert "honeypot" not in blob.lower()
    for h in HONEYPOTS:
        assert h["rationale"] not in blob


def test_t7_10_release_checksums_file_matches_returned_hashes(tmp_path):
    result = release.build(DISCOVERIES, CURATION, tmp_path / "out")
    checksums = json.loads((tmp_path / "out" / "CHECKSUMS.json").read_text())
    assert checksums["files"] == result["files"]
    assert checksums["release_sha256"] == result["release_sha256"]


def test_t7_10_release_csv_has_a_row_per_discovery(tmp_path):
    release.build(DISCOVERIES, CURATION, tmp_path / "out")
    lines = (tmp_path / "out" / "discoveries.csv").read_text().splitlines()
    assert len(lines) == 1 + len(DISCOVERIES)
