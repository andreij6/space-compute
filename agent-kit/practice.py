import json
import sys
from collections import defaultdict


def load_answers(path):
    with open(path) as f:
        data = json.load(f)
    return {row["subject_id"]: row["answers"] for row in data}


def score(agent_answers, key_answers):
    totals = defaultdict(int)
    correct = defaultdict(int)
    for subject_id, key in key_answers.items():
        given = agent_answers.get(subject_id, {})
        for question_id, expected in key.items():
            totals[question_id] += 1
            if given.get(question_id) == expected:
                correct[question_id] += 1
    return {q: correct[q] / totals[q] for q in totals}


def main():
    if len(sys.argv) != 3:
        print("usage: practice.py <agent_answers.json> <practice_answers.json>")
        sys.exit(1)
    agent = load_answers(sys.argv[1])
    key = load_answers(sys.argv[2])
    accuracy = score(agent, key)
    for question_id, acc in sorted(accuracy.items()):
        print(f"{question_id}: {acc:.2%}")
    overall = sum(accuracy.values()) / len(accuracy) if accuracy else 0.0
    print(f"overall: {overall:.2%}")


if __name__ == "__main__":
    main()
