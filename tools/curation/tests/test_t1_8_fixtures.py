import hashlib
import json
from pathlib import Path

import jsonschema
import pytest

from sc_curation import contact_sheet, fixtures

SCHEMA = json.loads((Path(__file__).parents[1] / "schema" / "sc-dossier-1.json").read_text())


@pytest.fixture(scope="module")
def built(tmp_path_factory):
    out = tmp_path_factory.mktemp("fixtures")
    summary = fixtures.build(out)
    return out, summary


def test_t1_8_fixture_set_has_50_subjects_10_gold_5_honeypots(built):
    _, summary = built
    assert summary == {"subjects": 50, "gold": 10, "honeypots": 5}


def test_t1_8_every_dossier_matches_schema_and_hashes(built):
    out, _ = built
    rows = [json.loads(l) for l in (out / "manifest_v1.jsonl").read_text().splitlines()]
    for row in rows:
        body = (out / row["dossier_url"]).read_bytes()
        assert hashlib.sha256(body).hexdigest() == row["dossier_sha256"]
        dossier = json.loads(body)
        jsonschema.validate(dossier, SCHEMA)
        folder = (out / row["dossier_url"]).parent
        files = [dossier["images"]["rgb"], dossier["images"]["rgb_sw"], dossier["images"]["segmentation"],
                 *dossier["images"]["fits"]]
        for f in files:
            assert hashlib.sha256((folder / f["url"]).read_bytes()).hexdigest() == f["sha256"]


def test_t1_8_gold_and_honeypot_status_never_leak_into_dossiers(built):
    out, _ = built
    for path in out.glob("v1/subjects/*/dossier.json"):
        text = path.read_text().lower()
        assert "gold" not in text and "honeypot" not in text
    bad = json.loads(next(out.glob("v1/subjects/*/dossier.json")).read_text())
    bad["gold"] = True
    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(bad, SCHEMA)


def test_t1_8_generation_is_deterministic(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    fixtures.build(a, count=6)
    fixtures.build(b, count=6)
    assert (a / "manifest_v1.jsonl").read_text() == (b / "manifest_v1.jsonl").read_text()


def test_t1_8_contact_sheet_renders(built, tmp_path):
    out, _ = built
    sheet = contact_sheet.build(out, tmp_path / "sheet.png")
    assert sheet.stat().st_size > 10_000
