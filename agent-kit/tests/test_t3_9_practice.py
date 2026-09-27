import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

import practice

FIXTURES = pathlib.Path(__file__).parent / "fixtures"


def test_t3_9_practice_per_question_accuracy():
    agent = practice.load_answers(FIXTURES / "agent_answers.json")
    key = practice.load_answers(FIXTURES / "practice_answers.json")
    accuracy = practice.score(agent, key)
    assert accuracy["shape"] == 2 / 3
    assert accuracy["discovery"] == 2 / 3


def test_t3_9_practice_missing_subject_counts_as_wrong():
    key = {1: {"shape": "featured"}, 2: {"shape": "smooth"}}
    agent = {1: {"shape": "featured"}}
    accuracy = practice.score(agent, key)
    assert accuracy["shape"] == 0.5
