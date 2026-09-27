import argparse
import json
from pathlib import Path

import numpy as np

from .render import STRETCH, asinh_rgb, fits_bytes, sha256

FIELD_CENTRES = {
    "ceers": (214.92, 52.88),
    "jades-gds": (53.16, -27.78),
    "jades-gdn": (189.23, 62.24),
    "primer-uds": (34.40, -5.20),
    "primer-cosmos": (150.12, 2.30),
    "abell2744": (3.58, -30.40),
}
FILTERS = [
    ("f115w", 1.154, 0.040), ("f150w", 1.501, 0.050), ("f200w", 1.990, 0.066),
    ("f277w", 2.786, 0.092), ("f356w", 3.563, 0.116), ("f444w", 4.421, 0.145),
]
KINDS = ["smooth", "disk", "spiral", "edgeon", "clumpy", "merger", "star", "arc", "red_compact"]
TRUTH = {
    "smooth": {"shape": "smooth", "clumps": "none", "merger": "none", "odd": "none"},
    "disk": {"shape": "featured", "edgeon": "no", "bar": "none", "spiral": "no", "clumps": "none", "merger": "none", "odd": "none"},
    "spiral": {"shape": "featured", "edgeon": "no", "bar": "none", "spiral": "yes", "clumps": "none", "merger": "none", "odd": "none"},
    "edgeon": {"shape": "featured", "edgeon": "yes", "clumps": "none", "merger": "none", "odd": "none"},
    "clumpy": {"shape": "featured", "edgeon": "no", "bar": "none", "spiral": "no", "clumps": "many", "merger": "none", "odd": "none"},
    "merger": {"shape": "featured", "edgeon": "no", "bar": "none", "spiral": "no", "clumps": "few", "merger": "major", "odd": "none"},
    "star": {"shape": "artifact"},
    "arc": {"shape": "compact", "odd": "arc"},
    "red_compact": {"shape": "compact", "odd": "red-compact"},
}
SIZE, PIXEL = 96, 0.04
ACK = ("This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope, "
       "obtained from the Mikulski Archive for Space Telescopes at STScI. Synthetic fixture data.")


def _profile(kind: str, rng: np.random.Generator, wave_um: float) -> np.ndarray:
    y, x = np.mgrid[0:SIZE, 0:SIZE] - SIZE / 2
    pa = rng.uniform(0, np.pi)
    xr, yr = x * np.cos(pa) + y * np.sin(pa), -x * np.sin(pa) + y * np.cos(pa)
    red = wave_um / 4.4
    if kind == "star":
        return 3.0 * np.exp(-(x**2 + y**2) / (2 * (1.2 * wave_um) ** 2))
    if kind == "red_compact":
        return 2.5 * red**3 * np.exp(-(x**2 + y**2) / 6.0)
    if kind == "arc":
        r, th = np.hypot(x, y + 30), np.arctan2(x, y + 30)
        return 0.8 * np.exp(-((r - 32) ** 2) / 6.0) * np.exp(-(th**2) / 0.25)
    q = 0.15 if kind == "edgeon" else rng.uniform(0.5, 0.9)
    rad = np.hypot(xr, yr / q)
    n = 4.0 if kind == "smooth" else 1.0
    img = 0.9 * np.exp(-2.0 * rad / 14.0) if n == 1 else 0.05 * np.exp(-7.67 * ((rad / 10.0) ** 0.25 - 1))
    if kind == "spiral":
        th = np.arctan2(yr, xr)
        img *= 0.35 + 0.9 * (np.cos(2 * (th - np.log(rad + 1) * 2.2)) > 0.2) * (rad > 3)
    if kind in ("clumpy", "merger"):
        for _ in range(5 if kind == "clumpy" else 1):
            cx, cy = rng.uniform(-14, 14, 2) if kind == "clumpy" else (22, -10)
            amp = 1.5 / red if kind == "clumpy" else 1.0
            img += amp * np.exp(-((x - cx) ** 2 + (y - cy) ** 2) / (4.0 if kind == "clumpy" else 40.0))
    return img


def build(outdir: Path, count: int = 50, gold: int = 10, honeypots: int = 5, seed: int = 20260927) -> dict:
    rng = np.random.default_rng(seed)
    fields = list(FIELD_CENTRES)
    subjects_dir = outdir / "v1" / "subjects"
    manifest, truth = [], {"gold": {}, "honeypots": {}}
    for i in range(count):
        sid = 900_000 + i
        kind = KINDS[i % len(KINDS)]
        field = fields[i % len(fields)]
        ra0, dec0 = FIELD_CENTRES[field]
        ra, dec = ra0 + rng.uniform(-0.05, 0.05), dec0 + rng.uniform(-0.05, 0.05)
        d = subjects_dir / str(sid)
        d.mkdir(parents=True, exist_ok=True)
        planes, fits_meta = {}, []
        for name, pivot, psf in FILTERS:
            img = _profile(kind, np.random.default_rng(seed + i), pivot) + rng.normal(0, 0.02, (SIZE, SIZE))
            planes[name] = img
            data = fits_bytes(img, {"FILTER": name.upper(), "CRVAL1": ra, "CRVAL2": dec, "CRPIX1": SIZE / 2,
                                    "CRPIX2": SIZE / 2, "CDELT1": -PIXEL / 3600, "CDELT2": PIXEL / 3600,
                                    "CTYPE1": "RA---TAN", "CTYPE2": "DEC--TAN", "BUNIT": "uJy/pix"},
                              weight=np.full((SIZE, SIZE), 1 / 0.02**2))
            (d / f"{name}.fits").write_bytes(data)
            fits_meta.append({"filter": name, "url": f"{name}.fits", "sha256": sha256(data), "pivot_um": pivot,
                              "psf_fwhm_arcsec": psf, "depth_5sigma_ab": 28.6, "exptime_s": 3100})
        seg = (planes["f444w"] > 0.1).astype(np.int32)
        seg_bytes = fits_bytes(seg, {"EXTNAME": "SEG"})
        (d / "seg.fits").write_bytes(seg_bytes)
        rgb = asinh_rgb(planes["f444w"], planes["f277w"], planes["f150w"])
        rgb_sw = asinh_rgb(planes["f200w"], planes["f150w"], planes["f115w"])
        (d / "rgb.png").write_bytes(rgb)
        (d / "rgb_sw.png").write_bytes(rgb_sw)
        z = float(np.round(rng.uniform(0.3, 9.0 if kind == "red_compact" else 4.0), 2))
        bands = {n: {"flux": float(np.round(planes[n].sum() * 0.01, 3)), "err": 0.03,
                     "mag_ab": float(np.round(23.9 - 2.5 * np.log10(max(planes[n].sum() * 0.01, 1e-3)), 2))}
                 for n, _, _ in FILTERS}
        dossier = {
            "schema": "sc-dossier/1",
            "subject_id": sid,
            "target": {"ra_deg": round(ra, 6), "dec_deg": round(dec, 6), "field": field,
                       "catalog": "fixture-v1", "catalog_id": i, "galactic_l_b": [0.0, 0.0], "on_mosaic_edge": False},
            "images": {
                "rgb": {"url": "rgb.png", "sha256": sha256(rgb), "channels": {"r": "f444w", "g": "f277w", "b": "f150w"},
                        "stretch": STRETCH, "size_arcsec": SIZE * PIXEL},
                "rgb_sw": {"url": "rgb_sw.png", "sha256": sha256(rgb_sw)},
                "fits": fits_meta,
                "segmentation": {"url": "seg.fits", "sha256": sha256(seg_bytes)},
                "pixel_scale_arcsec": PIXEL, "north_up": True,
            },
            "photometry": {"unit": "uJy", "aperture_arcsec": 0.5, "bands": bands},
            "redshift": {"z_phot": z, "z_phot_p16": round(z * 0.92, 2), "z_phot_p84": round(z * 1.08, 2),
                         "z_spec": None, "z_spec_grade": None, "has_spectrum": False,
                         "kpc_per_arcsec": None, "lookback_gyr": None, "cosmology": "Planck18"},
            "physical": {"log_mstar": None, "sfr_msun_yr": None, "rest_uv": None, "rest_vj": None, "source": "fixture"},
            "morphology_params": {"sersic_n": None, "r_e_arcsec": None, "axis_ratio": None, "pa_deg": None, "source": "fixture"},
            "lensing": {"magnification": 3.2 if kind == "arc" else None},
            "neighbours": [],
            "quality_flags": ["synthetic"],
            "provenance": {"programs": [{"pid": 0, "name": "fixture"}],
                           "credits": ["Space Compute synthetic fixtures"], "acknowledgment": ACK},
        }
        body = json.dumps(dossier, indent=1, sort_keys=True).encode()
        (d / "dossier.json").write_bytes(body)
        manifest.append({"subject_id": sid, "field": field, "kind": kind, "ra_deg": dossier["target"]["ra_deg"],
                         "dec_deg": dossier["target"]["dec_deg"],
                         "rgb_url": f"v1/subjects/{sid}/rgb.png", "rgb_sha256": sha256(rgb),
                         "dossier_url": f"v1/subjects/{sid}/dossier.json", "dossier_sha256": sha256(body)})
        if i < gold:
            truth["gold"][str(sid)] = TRUTH[kind]
        elif i < gold + honeypots:
            real = kind in ("arc", "merger", "red_compact")
            truth["honeypots"][str(sid)] = {"claimed_category": {"arc": "lensed_arc", "merger": "merger_interaction",
                                                                 "red_compact": "little_red_dot"}.get(kind, "ring"),
                                            "true_claim": real}
    (outdir / "manifest_v1.jsonl").write_text("".join(json.dumps(m, sort_keys=True) + "\n" for m in manifest))
    (outdir / "fixtures_truth.json").write_text(json.dumps(truth, indent=1, sort_keys=True))
    return {"subjects": len(manifest), "gold": len(truth["gold"]), "honeypots": len(truth["honeypots"])}


def main() -> None:
    p = argparse.ArgumentParser(description="Generate deterministic synthetic dossiers")
    p.add_argument("--out", default="../../target/fixtures")
    p.add_argument("--count", type=int, default=50)
    a = p.parse_args()
    print(json.dumps(build(Path(a.out), a.count)))


if __name__ == "__main__":
    main()
