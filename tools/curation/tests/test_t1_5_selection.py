import json

import numpy as np
from astropy.table import Table

from sc_curation import gold, select


def synthetic(field, n=400, seed=0, ra0=150.1, dec0=2.3):
    rng = np.random.default_rng(seed)
    phot = Table()
    phot["id"] = np.arange(1, n + 1)
    phot["ra"] = ra0 + rng.uniform(-0.01, 0.01, n)
    phot["dec"] = dec0 + rng.uniform(-0.01, 0.01, n)
    phot["flux_radius"] = rng.uniform(1.0, 8.0, n)
    phot["f444w_tot_corr"] = np.ones(n)
    flux = 10 ** ((23.9 - rng.uniform(20, 29, n)) / 2.5)
    phot["f444w_flux_aper_1"] = flux
    phot["f444w_fluxerr_aper_1"] = flux / rng.uniform(3, 60, n)
    phot["f277w_flux_aper_1"] = flux * rng.uniform(0.2, 1.2, n)
    for b in select.NIRCAM:
        if b not in ("f444w",):
            phot[f"{b}_fluxerr_aper_1"] = np.where(rng.uniform(0, 1, n) < 0.9, 0.01, np.nan)
    phot["f444w_fluxerr_aper_1"] = phot["f444w_fluxerr_aper_1"]
    z = Table()
    z["id"] = phot["id"]
    z["z_phot"] = rng.uniform(0.1, 9.0, n)
    for c in ["z160", "z840", "z_spec", "mass", "sfr", "restU", "restV", "restJ"]:
        z[c] = rng.uniform(0, 1, n)
    return select.prepare(phot, z, field, f"{field}-root", 0), phot


def test_t1_5_criteria_follow_spec_07():
    t, _ = synthetic("primer-cosmos")
    flags = select.classify(t)
    e = flags["eligible"]
    assert e.any()
    assert np.all(t["mag_f444w"][e] <= 27.0)
    assert np.all(t["snr_f444w"][e] >= 10.0)
    assert np.all(t["n_nircam"][e] >= 5)
    assert (e & flags["star"]).any()


def test_t1_5_stars_and_unknown_redshift_stay_in_the_pool(tmp_path):
    t, _ = synthetic("primer-cosmos", seed=5)
    t["z_phot"][:40] = np.nan
    flags = select.classify(t)
    assert (flags["eligible"][:40]).any()
    summary = select.run(tmp_path, select.Criteria(total=300), {}, {"primer-cosmos": t})
    assert summary["z_bins"].get("none", 0) > 0


def test_t1_5_quotas_fill_the_total_and_respect_capacity():
    q = select.quotas({"ceers": 5000, "abell2744": 100, "jades-gds": 50_000}, 20_000)
    assert sum(q.values()) == 20_000
    assert q["abell2744"] == 100
    assert q["jades-gds"] > q["ceers"] - 1


def test_t1_5_selection_is_deterministic_and_ids_unique(tmp_path):
    tables = {"primer-cosmos": synthetic("primer-cosmos")[0], "ceers": synthetic("ceers", seed=1)[0]}
    a = select.run(tmp_path / "a", select.Criteria(total=200), {}, tables)
    b = select.run(tmp_path / "b", select.Criteria(total=200), {}, tables)
    assert a == b
    rows = [json.loads(l) for l in (tmp_path / "a" / "selection_v1.jsonl").read_text().splitlines()]
    ids = [r["subject_id"] for r in rows]
    assert len(ids) == len(set(ids)) and 150 <= len(ids) <= 260
    assert set(a["z_bins"]) - {"none"}


def test_t1_5_gold_mapping_thresholds_and_z_guard(tmp_path):
    t, _ = synthetic("primer-cosmos", seed=3)
    flags = select.classify(t)
    i = int(np.flatnonzero(flags["eligible"] & (t["z_phot"] < 2))[0])
    j = int(np.flatnonzero(flags["eligible"] & (t["z_phot"] > 3))[0])
    def gz(ra, dec, smooth, votes, gid):
        return {"ID": gid, "RA": str(ra), "Dec": str(dec), "t00_smooth_or_featured_count": str(votes),
                "t00_smooth_or_featured_a0_smooth_frac": str(smooth),
                "t00_smooth_or_featured_a1_features_frac": str(1 - smooth),
                "t00_smooth_or_featured_a2_artifact_frac": "0"}
    rows = [
        gz(t["ra"][i], t["dec"][i], 0.9, 40, "COS_1"),
        gz(t["ra"][i] + 1e-3, t["dec"][i], 0.95, 40, "COS_2"),
        gz(t["ra"][j], t["dec"][j], 0.9, 40, "COS_3"),
        gz(t["ra"][i], t["dec"][i], 0.9, 10, "COS_4"),
    ]
    result = gold.run(tmp_path, {"primer-cosmos": t}, rows)
    g = json.loads((tmp_path / "gold_v1.json").read_text())
    matched = {k: v for k, v in g.items() if v["source"].startswith("gz-candels")}
    assert list(matched) == [str(int(t["subject_id"][i]))]
    assert matched[str(int(t["subject_id"][i]))]["answers"] == {"shape": "smooth"}
    assert result["summary"]["by_field"]["primer-cosmos"] >= 1


def test_t1_5_merger_none_alone_is_weak_gold():
    row = {"t16_merging_tidal_debris_count": "40", "t16_merging_tidal_debris_a3_neither_frac": "0.95"}
    answers = gold.gz_answers(row)
    assert answers == {"merger": "none"}
    assert gold.strength(answers) == "weak"
    assert gold.strength({"merger": "none", "shape": "smooth"}) == "strong"


def test_t1_5_committed_v1_selection_meets_acceptance():
    from collections import Counter
    from pathlib import Path

    root = Path(__file__).resolve().parents[3] / "data" / "curation" / "v1"
    rows = [json.loads(l) for l in (root / "selection_v1.jsonl").read_text().splitlines()]
    gold_rows = json.loads((root / "gold_v1.json").read_text())
    ids = {r["subject_id"] for r in rows}
    fields = Counter(r["field"] for r in rows)
    print(f"\n  ✓ {len(rows):,} subjects across {len(fields)} fields: {dict(fields)}")
    print(f"  ✓ {len(gold_rows):,} gold subjects ({sum(g['strength'] == 'strong' for g in gold_rows.values()):,} strong), all inside the selection")
    assert len(rows) == 5_000 and len(ids) == 5_000
    assert len(fields) == 6
    assert len(gold_rows) >= 750
    assert {int(k) for k in gold_rows} <= ids
    assert all(g["strength"] in ("strong", "weak") for g in gold_rows.values())
