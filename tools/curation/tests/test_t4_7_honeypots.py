import json
from collections import Counter, defaultdict
from pathlib import Path

from sc_curation import honeypots

ROOT = Path(__file__).resolve().parents[3]
GOLD = json.loads((ROOT / "data/curation/v1/gold_v1.json").read_text())
MANIFEST = honeypots.load_manifest(ROOT / "data/curation/v1/manifest_v1.jsonl")
PROTOCOL = json.loads((ROOT / "data/protocol/protocol_v1.json").read_text())
CATEGORY_IDS = {c["id"] for c in PROTOCOL["discovery_categories"]}
SPECS = honeypots.build(GOLD, MANIFEST)


def test_t4_7_count_is_120():
    assert len(SPECS) == 120


def test_t4_7_subjects_are_gold():
    gold_ids = {int(k) for k in GOLD}
    assert all(s["subject_id"] in gold_ids for s in SPECS)


def test_t4_7_categories_are_protocol_v1():
    assert all(s["category"] in CATEGORY_IDS for s in SPECS)


def test_t4_7_truth_is_agree_or_disagree():
    assert all(s["truth"] in ("Agree", "Disagree") for s in SPECS)


def test_t4_7_deterministic():
    assert SPECS == honeypots.build(GOLD, MANIFEST)


def test_t4_7_no_duplicate_subjects():
    ids = [s["subject_id"] for s in SPECS]
    assert len(ids) == len(set(ids))


def test_t4_7_truth_is_derived_from_the_subjects_gold_answers():
    for s in SPECS:
        answers = GOLD[str(s["subject_id"])]["answers"]
        claim = honeypots.claim_of(s)
        question, agree_answers = honeypots.CLAIMS[claim]["gold"]
        assert question in answers
        expected = "Agree" if answers[question] in agree_answers else "Disagree"
        assert s["truth"] == expected, (s, answers)


def test_t4_7_artifact_claims_on_artifact_gold_are_agree():
    for s in SPECS:
        if s["category"] == "artifact":
            is_artifact = GOLD[str(s["subject_id"])]["answers"]["shape"] == "artifact"
            assert (s["truth"] == "Agree") == is_artifact


def test_t4_7_agree_share_is_between_35_and_50_percent():
    share = sum(s["truth"] == "Agree" for s in SPECS) / len(SPECS)
    assert 0.35 <= share <= 0.50


def test_t4_7_always_disagree_does_not_score_well():
    disagree = sum(s["truth"] == "Disagree" for s in SPECS) / len(SPECS)
    assert disagree <= 0.65


def test_t4_7_no_rationale_template_maps_to_a_single_truth():
    by_text = defaultdict(set)
    for s in SPECS:
        by_text[(s["category"], s["rationale"])].add(s["truth"])
    assert all(v == {"Agree", "Disagree"} for v in by_text.values()), by_text


def test_t4_7_text_only_classifier_cannot_beat_base_rate_by_10pp():
    counts = defaultdict(Counter)
    for s in SPECS:
        counts[(s["category"], s["rationale"])][s["truth"]] += 1
    best = sum(max(c.values()) for c in counts.values()) / len(SPECS)
    base = max(Counter(s["truth"] for s in SPECS).values()) / len(SPECS)
    assert best - base <= 0.10


def test_t4_7_no_word_predicts_truth():
    words = defaultdict(Counter)
    for s in SPECS:
        for w in set(s["rationale"].lower().split()):
            words[w][s["truth"]] += 1
    for w, c in words.items():
        if sum(c.values()) >= 5:
            assert set(c) == {"Agree", "Disagree"}, (w, c)


def test_t4_7_rationales_carry_no_field_or_coordinate_suffix():
    for s in SPECS:
        assert "RA " not in s["rationale"] and " field," not in s["rationale"]


def test_t4_7_subjects_are_stratified_across_fields():
    fields = Counter(MANIFEST[s["subject_id"]]["field"] for s in SPECS)
    assert len(fields) >= 4
    assert max(fields.values()) / len(SPECS) <= 0.4


def test_t4_7_not_just_the_lowest_gold_ids():
    lowest = sorted(int(k) for k in GOLD)[: len(SPECS)]
    assert len(set(lowest) & {s["subject_id"] for s in SPECS}) < len(SPECS) // 2


def test_t4_7_committed_output_matches_build():
    committed = json.loads((ROOT / "data/curation/v1/honeypots_v1.json").read_text())
    assert committed == SPECS
