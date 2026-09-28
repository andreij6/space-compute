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


def test_t7_10_release_includes_manifest_selection_and_gold_metadata(tmp_path):
    release.build(DISCOVERIES, CURATION, tmp_path / "out")
    manifest = (tmp_path / "out" / "manifest.jsonl").read_text().splitlines()
    selection = (tmp_path / "out" / "selection.jsonl").read_text().splitlines()
    gold = (tmp_path / "out" / "gold.jsonl").read_text().splitlines()
    assert len(manifest) == 5000
    assert len(selection) == 5000
    assert len(gold) >= 750


def test_t7_10_release_checksums_file_matches_returned_hashes(tmp_path):
    result = release.build(DISCOVERIES, CURATION, tmp_path / "out")
    checksums = json.loads((tmp_path / "out" / "CHECKSUMS.json").read_text())
    assert checksums["files"] == result["files"]
    assert checksums["release_sha256"] == result["release_sha256"]


def test_t7_10_release_csv_has_a_row_per_discovery(tmp_path):
    release.build(DISCOVERIES, CURATION, tmp_path / "out")
    lines = (tmp_path / "out" / "discoveries.csv").read_text().splitlines()
    assert len(lines) == 1 + len(DISCOVERIES)
