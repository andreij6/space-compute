import os
import pathlib
import re

SKILL = pathlib.Path(
    os.environ.get(
        "SKILL_MD",
        pathlib.Path(__file__).resolve().parents[1] / "skills/space-compute-astronomer/SKILL.md",
    )
).read_text()


def test_t3_7_skill_has_no_runnable_add_operator_command():
    assert not re.search(r"icp canister call [^`\n]*add_operator", SKILL)
    assert "/connect" in SKILL


def test_t3_7_skill_forbids_key_export_and_key_file_reads():
    assert "never run `icp identity export`" in SKILL
    assert "identity key files" in SKILL


def test_t3_7_skill_quotes_urls_and_requires_https():
    assert not re.search(r"curl [^\n]*<(image|dossier)_url>(?!')", SKILL)
    assert "`https://`" in SKILL
    assert "http://127.0.0.1" in SKILL
