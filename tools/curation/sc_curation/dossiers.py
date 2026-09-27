import argparse
import io
import json
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
from astropy import units as u
from astropy.coordinates import SkyCoord
from astropy.cosmology import Planck18
from astropy.io import fits
from astropy.table import Table
from PIL import Image

from .render import sha256
from .sources import CACHE, DJA, FIELDS, phot_name, zout_name
from .stream import cut_mosaic

FILTERS = {"f115w": (1.154, 0.040), "f150w": (1.501, 0.050), "f200w": (1.990, 0.066),
           "f277w": (2.786, 0.092), "f356w": (3.563, 0.116), "f444w": (4.421, 0.145)}
SIZE = 390
PNG_SIZE = 512
STRETCH = {"type": "asinh", "a": 0.02, "min": -0.02, "max": 1.5}
PROGRAMS = {
    "ceers": [{"pid": 1345, "name": "CEERS"}],
    "jades-gds": [{"pid": 1180, "name": "JADES"}, {"pid": 1210, "name": "JADES"}, {"pid": 3215, "name": "JADES Origins Field"}],
    "jades-gdn": [{"pid": 1181, "name": "JADES"}],
    "primer-uds": [{"pid": 1837, "name": "PRIMER"}],
    "primer-cosmos": [{"pid": 1837, "name": "PRIMER"}],
    "abell2744": [{"pid": 2561, "name": "UNCOVER"}, {"pid": 3516, "name": "MegaScience"}],
}
CREDITS = ["Dawn JWST Archive v7 (Valentino et al. 2023; grizli, Brammer)", "EAZY photometric redshifts (Brammer et al. 2008)"]
ACK = ("This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope, obtained from the "
       "Mikulski Archive for Space Telescopes at the Space Telescope Science Institute, which is operated by the "
       "Association of Universities for Research in Astronomy, Inc., under NASA contract NAS 5-03127.")
KEEP_CARDS = ["PHOTFNU", "PIXAR_SR", "PHOTFLAM", "PHOTPLAM", "TELESCOP", "INSTRUME", "FILTER"]


def mosaic_url(root: str, band: str | None) -> str:
    return f"{DJA}{root}-{band}-clear_drc_sci.fits.gz" if band else f"{DJA}{root}-ir_seg.fits.gz"


def _cut_job(args):
    root, band, ra, dec, workdir = args
    out = Path(workdir) / f"{band or 'seg'}.npy"
    header, corners = cut_mosaic(mosaic_url(root, band), ra, dec, SIZE, out)
    (Path(workdir) / f"{band or 'seg'}.hdr").write_text(header.tostring())
    np.save(Path(workdir) / f"{band or 'seg'}.corners.npy", corners)
    return band


def cut_root(root: str, rows: list[dict], workdir: Path, workers: int) -> None:
    workdir.mkdir(parents=True, exist_ok=True)
    ra = np.array([r["ra_deg"] for r in rows])
    dec = np.array([r["dec_deg"] for r in rows])
    jobs = [(root, b, ra, dec, str(workdir)) for b in [*FILTERS, None] if not (workdir / f"{b or 'seg'}.done").exists()]
    with ProcessPoolExecutor(max_workers=workers) as pool:
        for band in pool.map(_cut_job, jobs):
            (workdir / f"{band or 'seg'}.done").write_text("ok")
            print(f"  cut {root} {band or 'seg'}", flush=True)


def cutout_header(mosaic: fits.Header, x0: int, y0: int, band: str | None) -> fits.Header:
    h = fits.Header()
    for k in ["CTYPE1", "CTYPE2", "CRVAL1", "CRVAL2", "CD1_1", "CD1_2", "CD2_1", "CD2_2", "RADESYS", "EQUINOX"]:
        if k in mosaic:
            h[k] = mosaic[k]
    h["CRPIX1"] = mosaic["CRPIX1"] - x0
    h["CRPIX2"] = mosaic["CRPIX2"] - y0
    for k in KEEP_CARDS:
        if k in mosaic:
            h[k] = mosaic[k]
    if band:
        h["FILTER"] = band.upper()
        h["BUNIT"] = "10 nJy / pixel (PHOTFNU=1e-8 Jy)" if mosaic.get("PHOTFNU") == 1e-8 else "see PHOTFNU"
    return h


def fits_bytes(data: np.ndarray, header: fits.Header, integer: bool = False) -> bytes:
    arr = data.astype(np.int32) if integer else np.nan_to_num(data, nan=0.0).astype(np.float32)
    hdu = fits.CompImageHDU(arr, header=header, compression_type="RICE_1", quantize_level=16 if not integer else None)
    buf = io.BytesIO()
    fits.HDUList([fits.PrimaryHDU(), hdu]).writeto(buf, checksum=False)
    return buf.getvalue()


def rgb_png(r: np.ndarray, g: np.ndarray, b: np.ndarray, stretch: dict = STRETCH) -> bytes:
    lo, hi, a = stretch["min"], stretch["max"], stretch["a"]
    planes = []
    for band in (r, g, b):
        x = np.clip((np.nan_to_num(band, nan=0.0) - lo) / (hi - lo), 0, 1)
        planes.append(np.round(np.arcsinh(x / a) / np.arcsinh(1 / a) * 255).astype(np.uint8))
    img = Image.fromarray(np.flipud(np.dstack(planes)), mode="RGB").resize((PNG_SIZE, PNG_SIZE), Image.BILINEAR)
    buf = io.BytesIO()
    img.save(buf, format="PNG", compress_level=6)
    return buf.getvalue()


REF_SCALE = 0.04


def common_grid(planes: dict, scales: dict) -> tuple[dict, float]:
    fine = min(scales.values())
    out = {}
    for band, data in planes.items():
        factor = int(round(scales[band] / fine))
        x = np.asarray(data, dtype=np.float32) * (REF_SCALE / scales[band]) ** 2
        if factor > 1:
            n = SIZE // factor
            lo = SIZE // 2 - n // 2
            x = np.repeat(np.repeat(x[lo:lo + n, lo:lo + n], factor, axis=0), factor, axis=1)
            x = np.pad(x, ((0, SIZE - x.shape[0]), (0, SIZE - x.shape[1])), constant_values=np.nan)
        out[band] = x
    return out, round(SIZE * fine, 2)


def _num(v, nd=3):
    try:
        f = float(v)
    except (TypeError, ValueError):
        return None
    return round(f, nd) if np.isfinite(f) else None


def catalog_rows(root: str, ids: list[int]) -> tuple[dict, dict, Table]:
    phot = Table.read(CACHE / phot_name(root), memmap=True)
    zout = Table.read(CACHE / zout_name(root), memmap=True)
    pidx = {int(i): k for k, i in enumerate(phot["id"])}
    zidx = {int(i): k for k, i in enumerate(zout["id"])}
    return {i: phot[pidx[i]] for i in ids if i in pidx}, {i: zout[zidx[i]] for i in ids if i in zidx}, phot


def build_dossier(row: dict, p, z, neighbours: list, files: dict, edge: bool) -> dict:
    corr = float(p["f444w_tot_corr"]) if "f444w_tot_corr" in p.colnames else 1.0
    bands = {}
    for band in FILTERS:
        f, e = p[f"{band}_flux_aper_1"] if f"{band}_flux_aper_1" in p.colnames else None, p[f"{band}_fluxerr_aper_1"] if f"{band}_fluxerr_aper_1" in p.colnames else None
        flux = _num(f * corr if f is not None else None)
        bands[band] = {"flux": flux, "err": _num(e * corr if e is not None else None),
                       "mag_ab": round(23.9 - 2.5 * np.log10(flux), 3) if flux and flux > 0 else None}
    zp = _num(z["z_phot"]) if z is not None else None
    zs = _num(z["z_spec"]) if z is not None and float(z["z_spec"]) > 0 else None
    zz = zs or (zp if zp and zp > 0 else None)
    mass = float(z["mass"]) if z is not None else float("nan")
    rest = lambda a, b: _num(-2.5 * np.log10(float(z[a]) / float(z[b]))) if z is not None and float(z[a]) > 0 and float(z[b]) > 0 else None
    c = SkyCoord(row["ra_deg"] * u.deg, row["dec_deg"] * u.deg)
    a_img, b_img = float(p["a_image"]), float(p["b_image"])
    pix = files["pixel_scale_arcsec"]
    return {
        "schema": "sc-dossier/1",
        "subject_id": row["subject_id"],
        "target": {"ra_deg": row["ra_deg"], "dec_deg": row["dec_deg"], "field": row["field"], "catalog": f"{row['root']}-fix_phot",
                   "catalog_id": row["catalog_id"], "galactic_l_b": [round(c.galactic.l.deg, 4), round(c.galactic.b.deg, 4)],
                   "on_mosaic_edge": edge},
        "images": {"rgb": files["rgb"], "rgb_sw": files["rgb_sw"], "fits": files["fits"], "segmentation": files["seg"],
                   "pixel_scale_arcsec": pix, "north_up": True},
        "photometry": {"unit": "uJy", "aperture_arcsec": 0.5, "bands": bands},
        "redshift": {"z_phot": zp, "z_phot_p16": _num(z["z160"]) if z is not None else None, "z_phot_p84": _num(z["z840"]) if z is not None else None,
                     "z_spec": zs, "z_spec_grade": None, "has_spectrum": zs is not None,
                     "kpc_per_arcsec": round(float(Planck18.kpc_proper_per_arcmin(zz).value) / 60, 3) if zz else None,
                     "lookback_gyr": round(float(Planck18.lookback_time(zz).value), 3) if zz else None, "cosmology": "Planck18"},
        "physical": {"log_mstar": round(float(np.log10(mass)), 2) if mass > 0 else None, "sfr_msun_yr": _num(z["sfr"]) if z is not None else None,
                     "rest_uv": rest("restU", "restV"), "rest_vj": rest("restV", "restJ"), "source": f"eazy ({row['root']})"},
        "morphology_params": {"sersic_n": None, "r_e_arcsec": _num(float(p["flux_radius"]) * 0.04), "axis_ratio": _num(b_img / a_img) if a_img > 0 else None,
                              "pa_deg": _num(np.degrees(float(p["theta_image"]))), "source": "DJA phot (SExtractor-like moments)"},
        "lensing": {"magnification": None},
        "neighbours": neighbours,
        "quality_flags": ["mosaic_edge"] if edge else ["none"],
        "provenance": {"programs": PROGRAMS[row["field"]], "credits": CREDITS, "acknowledgment": ACK},
    }


def assemble_root(root: str, rows: list[dict], workdir: Path, bucket: Path) -> list[dict]:
    headers = {b: fits.Header.fromstring((workdir / f"{b}.hdr").read_text()) for b in [*FILTERS, "seg"]}
    cubes = {b: np.load(workdir / f"{b}.npy", mmap_mode="r") for b in [*FILTERS, "seg"]}
    corners = {b: np.load(workdir / f"{b}.corners.npy") for b in [*FILTERS, "seg"]}
    phot_rows, z_rows, phot = catalog_rows(root, [r["catalog_id"] for r in rows])
    cat = SkyCoord(np.asarray(phot["ra"]) * u.deg, np.asarray(phot["dec"]) * u.deg)
    pix = round(abs(float(headers["f444w"]["CD1_1"])) * 3600, 5)
    scales = {b: abs(float(headers[b]["CD1_1"])) * 3600 for b in FILTERS}
    manifest = []
    for k, row in enumerate(rows):
        p = phot_rows.get(row["catalog_id"])
        if p is None:
            continue
        z = z_rows.get(row["catalog_id"])
        d = bucket / "v1" / "subjects" / str(row["subject_id"])
        d.mkdir(parents=True, exist_ok=True)
        planes, fits_meta = {}, []
        for band, (pivot, psf) in FILTERS.items():
            x0, y0 = corners[band][k]
            data = np.asarray(cubes[band][k])
            planes[band] = data
            body = fits_bytes(data, cutout_header(headers[band], int(x0), int(y0), band))
            (d / f"{band}.fits").write_bytes(body)
            fits_meta.append({"filter": band, "url": f"{band}.fits", "sha256": sha256(body), "pivot_um": pivot,
                              "psf_fwhm_arcsec": psf, "depth_5sigma_ab": None, "exptime_s": None})
        sx0, sy0 = corners["seg"][k]
        seg = fits_bytes(np.nan_to_num(np.asarray(cubes["seg"][k]), nan=0), cutout_header(headers["seg"], int(sx0), int(sy0), None), integer=True)
        (d / "seg.fits").write_bytes(seg)
        grid, fov = common_grid(planes, scales)
        rgb = rgb_png(grid["f444w"], grid["f277w"], grid["f150w"])
        rgb_sw = rgb_png(grid["f200w"], grid["f150w"], grid["f115w"])
        (d / "rgb.png").write_bytes(rgb)
        (d / "rgb_sw.png").write_bytes(rgb_sw)
        f444 = planes["f444w"]
        edge = bool(np.mean(~np.isfinite(f444) | (f444 == 0)) > 0.05)
        c = SkyCoord(row["ra_deg"] * u.deg, row["dec_deg"] * u.deg)
        sep = c.separation(cat).arcsec
        near = np.flatnonzero((sep > 0.05) & (sep < 5.0))
        near = near[np.argsort(sep[near])][:5]
        neighbours = [{"catalog_id": int(phot["id"][i]), "sep_arcsec": round(float(sep[i]), 2)} for i in near]
        files = {"rgb": {"url": "rgb.png", "sha256": sha256(rgb), "channels": {"r": "f444w", "g": "f277w", "b": "f150w"},
                         "stretch": STRETCH, "size_arcsec": fov},
                 "rgb_sw": {"url": "rgb_sw.png", "sha256": sha256(rgb_sw)}, "fits": fits_meta,
                 "seg": {"url": "seg.fits", "sha256": sha256(seg)}, "pixel_scale_arcsec": pix}
        body = json.dumps(build_dossier(row, p, z, neighbours, files, edge), indent=1, sort_keys=True).encode()
        (d / "dossier.json").write_bytes(body)
        manifest.append({"subject_id": row["subject_id"], "field": row["field"], "ra_deg": row["ra_deg"], "dec_deg": row["dec_deg"],
                         "rgb_url": f"v1/subjects/{row['subject_id']}/rgb.png", "rgb_sha256": sha256(rgb),
                         "dossier_url": f"v1/subjects/{row['subject_id']}/dossier.json", "dossier_sha256": sha256(body),
                         "data_version": 1, "on_mosaic_edge": edge})
    return manifest


def main() -> None:
    ap = argparse.ArgumentParser(description="Cut, render and package JWST subject dossiers")
    ap.add_argument("--selection", default="../../data/curation/v1/selection_v1.jsonl")
    ap.add_argument("--bucket", default="../../target/bucket")
    ap.add_argument("--work", default=str(CACHE.parent / "cutouts"))
    ap.add_argument("--roots", nargs="*")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--workers", type=int, default=7)
    a = ap.parse_args()
    rows = [json.loads(l) for l in Path(a.selection).read_text().splitlines()]
    by_root = defaultdict(list)
    for r in rows:
        by_root[r["root"]].append(r)
    roots = a.roots or [root for f in FIELDS.values() for root, _ in f]
    bucket = Path(a.bucket)
    for root in roots:
        todo = by_root[root][: a.limit] if a.limit else by_root[root]
        work = Path(a.work) / (root + (f"-limit{a.limit}" if a.limit else ""))
        print(f"{root}: {len(todo)} subjects", flush=True)
        cut_root(root, todo, work, a.workers)
        part = assemble_root(root, todo, work, bucket)
        (bucket / f"manifest_v1.{root}.jsonl").write_text("".join(json.dumps(m, sort_keys=True) + "\n" for m in part))
        print(f"  assembled {len(part)} dossiers", flush=True)
    parts = sorted(bucket.glob("manifest_v1.*.jsonl"))
    merged = sorted((json.loads(l) for p in parts for l in p.read_text().splitlines()), key=lambda m: m["subject_id"])
    (bucket / "manifest_v1.jsonl").write_text("".join(json.dumps(m, sort_keys=True) + "\n" for m in merged))
    print(f"manifest_v1.jsonl: {len(merged)} dossiers", flush=True)


if __name__ == "__main__":
    main()
