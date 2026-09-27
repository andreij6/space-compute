import json
from pathlib import Path

from sc_curation import honeypots

ROOT = Path(__file__).resolve().parents[3]
GOLD = json.loads((ROOT / "data/curation/v1/gold_v1.json").read_text())
MANIFEST = honeypots.load_manifest(ROOT / "data/curation/v1/manifest_v1.jsonl")
PROTOCOL = json.loads((ROOT / "data/protocol/protocol_v1.json").read_text())
CATEGORY_IDS = {c["id"] for c in PROTOCOL["discovery_categories"]}


def test_t4_7_count_is_120():
    assert len(honeypots.build(GOLD, MANIFEST)) == 120


def test_t4_7_subjects_are_gold():
    gold_ids = {int(k) for k in GOLD}
    specs = honeypots.build(GOLD, MANIFEST)
    assert all(s["subject_id"] in gold_ids for s in specs)


def test_t4_7_categories_are_protocol_v1():
    specs = honeypots.build(GOLD, MANIFEST)
    assert all(s["category"] in CATEGORY_IDS for s in specs)


def test_t4_7_truth_is_agree_or_disagree():
    specs = honeypots.build(GOLD, MANIFEST)
    assert all(s["truth"] in ("Agree", "Disagree") for s in specs)


def test_t4_7_deterministic():
    assert honeypots.build(GOLD, MANIFEST) == honeypots.build(GOLD, MANIFEST)


def test_t4_7_true_claims_are_supported_by_gold():
    specs = honeypots.build(GOLD, MANIFEST)
    for s in specs:
        if s["truth"] == "Agree":
            assert GOLD[str(s["subject_id"])]["answers"].get("merger") == "major"


def test_t4_7_false_claims_lack_gold_support():
    specs = honeypots.build(GOLD, MANIFEST)
    for s in specs:
        if s["truth"] == "Disagree" and s["category"] == "merger_interaction":
            assert GOLD[str(s["subject_id"])]["answers"].get("merger") != "major"


def test_t4_7_no_duplicate_subjects():
    specs = honeypots.build(GOLD, MANIFEST)
    ids = [s["subject_id"] for s in specs]
    assert len(ids) == len(set(ids))


def test_t4_7_committed_output_matches_build():
    committed = json.loads((ROOT / "data/curation/v1/honeypots_v1.json").read_text())
    assert committed == honeypots.build(GOLD, MANIFEST)
