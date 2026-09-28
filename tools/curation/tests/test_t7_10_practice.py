import json
from pathlib import Path

import pytest

from sc_curation import practice

ROOT = Path(__file__).resolve().parents[3]
CURATION = ROOT / "data" / "curation" / "v1"
PROTOCOL = json.loads((ROOT / "data/protocol/protocol_v1.json").read_text())
QUESTION_OPTIONS = {q["id"]: {a["id"] for a in q["answers"]} for q in PROTOCOL["questions"]}

SELECTION_IDS = {json.loads(l)["subject_id"] for l in (CURATION / "selection_v1.jsonl").read_text().splitlines()}
GOLD_IDS = {int(k) for k in json.loads((CURATION / "gold_v1.json").read_text())}
PRACTICE_ROWS = [json.loads(l) for l in (CURATION / "practice_v1.jsonl").read_text().splitlines()]
PRACTICE_ANSWERS = json.loads((CURATION / "practice_answers_v1.json").read_text())
PRACTICE_MANIFEST = [json.loads(l) for l in (CURATION / "practice_manifest_v1.jsonl").read_text().splitlines()]


def test_t7_10_choose_is_deterministic_and_disjoint_from_selection_and_gold():
    pool = {i: {"answers": {"shape": "smooth"}, "field": "jades-gds"} for i in range(1, 1001)}
    selection_ids = set(range(1, 501))
    gold_ids = {5, 6, 7}
    a = practice.choose(pool, selection_ids, gold_ids, count=200)
    b = practice.choose(pool, selection_ids, gold_ids, count=200)
    assert a == b
    assert len(a) == 200
    assert set(a).isdisjoint(selection_ids)
    assert set(a).isdisjoint(gold_ids)
    assert a == sorted(a)


def test_t7_10_choose_raises_when_the_non_overlapping_labelled_pool_is_too_small():
    pool = {i: {"answers": {}, "field": "f"} for i in range(1, 50)}
    with pytest.raises(ValueError):
        practice.choose(pool, set(), set(), count=200)


def test_t7_10_labelled_pool_already_includes_weak_matches_not_just_gold_strength():
    pool = {
        1: {"answers": {"merger": "none"}, "strength": "weak", "field": "jades-gds"},
        2: {"answers": {"shape": "smooth"}, "strength": "strong", "field": "jades-gds"},
    }
    chosen = practice.choose(pool, set(), set(), count=2)
    assert set(chosen) == {1, 2}


def test_t7_10_committed_practice_set_has_200_unique_subjects():
    ids = {r["subject_id"] for r in PRACTICE_ROWS}
    assert len(PRACTICE_ROWS) == 200
    assert len(ids) == 200
    assert {a["subject_id"] for a in PRACTICE_ANSWERS} == ids


def test_t7_10_committed_practice_set_never_overlaps_selection_or_gold():
    ids = {r["subject_id"] for r in PRACTICE_ROWS}
    assert ids.isdisjoint(SELECTION_IDS)
    assert ids.isdisjoint(GOLD_IDS)


def test_t7_10_committed_practice_answers_are_valid_protocol_v1():
    assert len(PRACTICE_ANSWERS) == 200
    for row in PRACTICE_ANSWERS:
        assert row["answers"]
        for question, answer in row["answers"].items():
            assert question in QUESTION_OPTIONS
            assert answer in QUESTION_OPTIONS[question]


def test_t7_10_committed_practice_manifest_matches_the_answer_key_subjects():
    manifest_ids = {m["subject_id"] for m in PRACTICE_MANIFEST}
    answer_ids = {a["subject_id"] for a in PRACTICE_ANSWERS}
    assert manifest_ids == answer_ids


def test_t7_10_agent_kit_practice_scores_the_committed_answer_key_perfectly():
    import importlib.util

    spec = importlib.util.spec_from_file_location("agent_kit_practice", ROOT / "agent-kit" / "practice.py")
    agent_kit_practice = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(agent_kit_practice)
    key = {row["subject_id"]: row["answers"] for row in PRACTICE_ANSWERS}
    accuracy = agent_kit_practice.score(key, key)
    assert accuracy and all(v == 1.0 for v in accuracy.values())
