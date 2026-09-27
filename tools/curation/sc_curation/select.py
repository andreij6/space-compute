import argparse
import json
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from astropy.table import Table, vstack

from .sources import CACHE, FIELDS, phot_name, zout_name

NIRCAM = ["f090w", "f115w", "f150w", "f200w", "f277w", "f356w", "f410m", "f444w"]
Z_BINS = [(0.0, 1.0, 0.25), (1.0, 3.0, 0.35), (3.0, 6.0, 0.25), (6.0, 99.0, 0.15)]
FIELD_WEIGHT = {"abell2744": 1.5}
MAG_LIMIT, SNR_LIMIT, MIN_BANDS = 27.0, 10.0, 5
POINT_RADIUS_PX, STAR_MAG, STAR_MAX_COLOR = 2.3, 24.0, 0.3
MAX_RED_COMPACT_PER_FIELD = 250
GOLD_SHARE = 0.25
FIELD_INDEX = {f: i + 1 for i, f in enumerate(FIELDS)}


@dataclass
class Criteria:
    total: int = 20_000
    seed: int = 20260927


def _mag(flux_ujy):
    with np.errstate(divide="ignore", invalid="ignore"):
        return np.where(flux_ujy > 0, 23.9 - 2.5 * np.log10(flux_ujy), np.nan)


def prepare(phot: Table, zout: Table, field: str, root: str, root_index: int) -> Table:
    z = zout[["id", "z_phot", "z160", "z840", "z_spec", "mass", "sfr", "restU", "restV", "restJ"]]
    t = Table()
    t["catalog_id"] = np.asarray(phot["id"], dtype=np.int64)
    t["ra"], t["dec"] = np.asarray(phot["ra"], float), np.asarray(phot["dec"], float)
    t["flux_radius_px"] = np.asarray(phot["flux_radius"], float)
    corr = np.asarray(phot["f444w_tot_corr"], float) if "f444w_tot_corr" in phot.colnames else 1.0
    f444 = np.asarray(phot["f444w_flux_aper_1"], float) * corr
    e444 = np.asarray(phot["f444w_fluxerr_aper_1"], float) * corr
    t["mag_f444w"] = _mag(f444)
    with np.errstate(divide="ignore", invalid="ignore"):
        t["snr_f444w"] = np.where(e444 > 0, f444 / e444, 0.0)
    bands = np.zeros(len(phot), dtype=int)
    for b in NIRCAM:
        col = f"{b}_fluxerr_aper_1"
        if col in phot.colnames:
            err = np.asarray(phot[col], float)
            bands += (np.isfinite(err) & (err > 0)).astype(int)
    t["n_nircam"] = bands
    f277 = np.asarray(phot["f277w_flux_aper_1"], float) if "f277w_flux_aper_1" in phot.colnames else np.full(len(phot), np.nan)
    t["color_277_444"] = _mag(f277) - _mag(np.asarray(phot["f444w_flux_aper_1"], float))
    zmap = {int(i): k for k, i in enumerate(z["id"])}
    idx = np.array([zmap.get(int(i), -1) for i in t["catalog_id"]])
    for col in ["z_phot", "z160", "z840", "z_spec", "mass", "sfr", "restU", "restV", "restJ"]:
        vals = np.asarray(z[col], float)
        t[col] = np.where(idx >= 0, vals[np.clip(idx, 0, None)], np.nan)
    t["field"] = field
    t["root"] = root
    t["subject_id"] = FIELD_INDEX[field] * 10_000_000 + root_index * 1_000_000 + t["catalog_id"]
    return t


def classify(t: Table) -> dict:
    ok_mag = np.isfinite(t["mag_f444w"]) & (t["mag_f444w"] <= MAG_LIMIT)
    ok_snr = t["snr_f444w"] >= SNR_LIMIT
    ok_bands = t["n_nircam"] >= MIN_BANDS
    point = (t["flux_radius_px"] > 0) & (t["flux_radius_px"] < POINT_RADIUS_PX)
    star = point & (t["mag_f444w"] < STAR_MAG) & (np.nan_to_num(t["color_277_444"], nan=9.0) < STAR_MAX_COLOR)
    eligible = ok_mag & ok_snr & ok_bands
    red_compact = eligible & (t["color_277_444"] > 1.0) & (t["flux_radius_px"] < 2.5) & (t["z_phot"] > 4)
    return {"eligible": eligible, "star": star & ok_bands, "red_compact": red_compact}


def load_field(field: str) -> Table:
    parts = []
    for k, (root, _) in enumerate(FIELDS[field]):
        phot = Table.read(CACHE / phot_name(root), memmap=True)
        zout = Table.read(CACHE / zout_name(root), memmap=True)
        parts.append(prepare(phot, zout, field, root, k))
    return vstack(parts)


def quotas(eligible_counts: dict, total: int) -> dict:
    weights = {f: FIELD_WEIGHT.get(f, 1.0) for f in eligible_counts}
    remaining, open_fields, out = total, set(eligible_counts), {f: 0 for f in eligible_counts}
    while remaining > 0 and open_fields:
        wsum = sum(weights[f] for f in open_fields)
        share = {f: int(remaining * weights[f] / wsum) for f in open_fields}
        progressed = False
        for f in sorted(open_fields):
            take = min(share[f], eligible_counts[f] - out[f])
            out[f] += take
            progressed |= take > 0
        remaining = total - sum(out.values())
        open_fields = {f for f in open_fields if out[f] < eligible_counts[f]}
        if not progressed:
            for f in sorted(open_fields)[:remaining]:
                out[f] += 1
            remaining = total - sum(out.values())
    return out


def choose(t: Table, flags: dict, quota: int, forced: set, rng: np.random.Generator) -> tuple[np.ndarray, dict]:
    reasons = {}
    picked = []
    ids = np.asarray(t["subject_id"])
    for i in np.flatnonzero(np.isin(ids, list(forced))):
        picked.append(i)
        reasons[int(ids[i])] = "gold"
    reds = np.flatnonzero(flags["red_compact"])
    for i in rng.permutation(reds)[:MAX_RED_COMPACT_PER_FIELD]:
        picked.append(i)
        reasons.setdefault(int(ids[i]), "red_compact")
    taken = set(picked)
    left = max(quota - len(taken), 0)
    pool = np.flatnonzero(flags["eligible"])
    pool = np.array([i for i in pool if i not in taken], dtype=int)
    z = np.asarray(t["z_phot"])[pool] if len(pool) else np.array([])
    unknown = pool[~(np.isfinite(z) & (z > 0))]
    n_unknown = int(left * len(unknown) / max(len(pool), 1))
    known = left - n_unknown
    bin_rows = [pool[(z >= lo) & (z < hi)] for lo, hi, _ in Z_BINS] + [unknown]
    targets = [int(known * w) for _, _, w in Z_BINS]
    targets[1] += known - sum(targets)
    targets.append(n_unknown)
    spill = 0
    for rows, target in sorted(zip(bin_rows, targets), key=lambda p: len(p[0])):
        n = min(len(rows), target + spill)
        spill = target + spill - n
        for i in rng.permutation(rows)[:n]:
            picked.append(i)
            reasons.setdefault(int(ids[i]), "stratified")
    picked = np.array(sorted(set(picked)), dtype=int)
    return picked, reasons


def z_bin(z: float) -> str:
    for lo, hi, _ in Z_BINS:
        if lo <= z < hi:
            return f"{lo:g}-{hi:g}" if hi < 99 else f">{lo:g}"
    return "none"


def run(out: Path, criteria: Criteria, gold_ids: dict | None = None, tables: dict | None = None) -> dict:
    rng = np.random.default_rng(criteria.seed)
    tables = tables or {f: load_field(f) for f in FIELDS}
    flags = {f: classify(t) for f, t in tables.items()}
    counts = {f: int(flags[f]["eligible"].sum()) for f in tables}
    q = quotas(counts, criteria.total)
    gold_ids = gold_ids or {}
    rows, summary = [], {"fields": {}, "z_bins": {}, "reasons": {}}
    for f, t in tables.items():
        forced = sorted(int(s) for s in gold_ids.get(f, []))
        forced = set(rng.permutation(forced)[:int(q[f] * GOLD_SHARE)].tolist()) if forced else set()
        picked, reasons = choose(t, flags[f], q[f], forced, rng)
        for i in picked:
            r = t[int(i)]
            sid = int(r["subject_id"])
            rec = {
                "subject_id": sid, "field": f, "root": str(r["root"]), "catalog_id": int(r["catalog_id"]),
                "ra_deg": round(float(r["ra"]), 7), "dec_deg": round(float(r["dec"]), 7),
                "z_phot": None if not np.isfinite(r["z_phot"]) else round(float(r["z_phot"]), 3),
                "mag_f444w": None if not np.isfinite(r["mag_f444w"]) else round(float(r["mag_f444w"]), 3),
                "flux_radius_px": round(float(r["flux_radius_px"]), 3),
                "reason": reasons.get(sid, "stratified"),
            }
            rows.append(rec)
            summary["fields"][f] = summary["fields"].get(f, 0) + 1
            zb = z_bin(rec["z_phot"]) if rec["z_phot"] is not None else "none"
            summary["z_bins"][zb] = summary["z_bins"].get(zb, 0) + 1
            summary["reasons"][rec["reason"]] = summary["reasons"].get(rec["reason"], 0) + 1
    rows.sort(key=lambda r: r["subject_id"])
    out.mkdir(parents=True, exist_ok=True)
    (out / "selection_v1.jsonl").write_text("".join(json.dumps(r, sort_keys=True) + "\n" for r in rows))
    summary["total"] = len(rows)
    summary["eligible"] = counts
    summary["quotas"] = q
    (out / "selection_summary_v1.json").write_text(json.dumps(summary, indent=1, sort_keys=True))
    return summary


def main() -> None:
    p = argparse.ArgumentParser(description="Select JWST subjects from DJA v7 catalogs")
    p.add_argument("--out", default="../../target/curation/v1")
    p.add_argument("--total", type=int, default=20_000)
    a = p.parse_args()
    from . import gold
    tables = {f: load_field(f) for f in FIELDS}
    gold_result = gold.run(Path(a.out), tables)
    summary = run(Path(a.out), Criteria(total=a.total), gold_result["ids_by_field"], tables)
    picked = {json.loads(l)["subject_id"] for l in (Path(a.out) / "selection_v1.jsonl").read_text().splitlines()}
    gold_path = Path(a.out) / "gold_v1.json"
    kept = {k: v for k, v in json.loads(gold_path.read_text()).items() if int(k) in picked}
    gold_path.write_text(json.dumps(kept, indent=1, sort_keys=True))
    gold_result["summary"]["in_selection"] = len(kept)
    print(json.dumps({"selection": summary, "gold": gold_result["summary"]}, indent=1))


if __name__ == "__main__":
    main()
