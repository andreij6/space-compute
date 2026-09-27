import io

import numpy as np
from astropy.io import fits
from astropy.wcs import WCS
from PIL import Image

from sc_curation import dossiers


def mosaic_header():
    w = WCS(naxis=2)
    w.wcs.ctype = ["RA---TAN", "DEC--TAN"]
    w.wcs.crval = [214.92, 52.87]
    w.wcs.crpix = [18432.0, 6144.0]
    w.wcs.cd = [[-7.142e-6, 0], [0, 7.142e-6]]
    h = w.to_header()
    h["CD1_1"], h["CD1_2"], h["CD2_1"], h["CD2_2"] = -7.142e-6, 0.0, 0.0, 7.142e-6
    for k in ["PC1_1", "PC1_2", "PC2_1", "PC2_2", "CDELT1", "CDELT2"]:
        h.remove(k, ignore_missing=True)
    h["PHOTFNU"], h["PIXAR_SR"] = 1e-8, 2.29e-14
    return h


def test_t1_7_cutout_wcs_points_at_the_target():
    h = mosaic_header()
    big = WCS(h)
    x, y = 20000.3, 7000.7
    ra, dec = big.all_pix2world(x, y, 0)
    x0, y0 = int(round(x)) - dossiers.SIZE // 2, int(round(y)) - dossiers.SIZE // 2
    small = WCS(dossiers.cutout_header(h, x0, y0, "f444w"))
    cra, cdec = small.all_pix2world(x - x0, y - y0, 0)
    assert abs(cra - ra) * 3600 < 1e-6 and abs(cdec - dec) * 3600 < 1e-6


def test_t1_7_rice_fits_round_trip_keeps_signal():
    rng = np.random.default_rng(0)
    data = rng.normal(0, 0.05, (dossiers.SIZE, dossiers.SIZE)).astype(np.float32)
    data[190:200, 190:200] += 5
    body = dossiers.fits_bytes(data, dossiers.cutout_header(mosaic_header(), 0, 0, "f444w"))
    with fits.open(io.BytesIO(body)) as hdul:
        back = hdul[1].data
        assert hdul[1].header["FILTER"] == "F444W" and hdul[1].header["PHOTFNU"] == 1e-8
    assert np.allclose(back, data, atol=0.02)
    assert len(body) < data.nbytes / 2


def test_t1_7_rgb_png_is_512_square():
    plane = np.zeros((dossiers.SIZE, dossiers.SIZE), np.float32)
    img = Image.open(io.BytesIO(dossiers.rgb_png(plane, plane, plane)))
    assert img.size == (512, 512) and img.mode == "RGB"


def test_t1_7_committed_manifest_and_qa_meet_acceptance():
    import json
    from pathlib import Path

    root = Path(__file__).resolve().parents[3] / "data" / "curation" / "v1"
    manifest = [json.loads(l) for l in (root / "manifest_v1.jsonl").read_text().splitlines()]
    qa = json.loads((root / "qa_v1.json").read_text())
    verify = json.loads((root / "verify_v1.json").read_text())
    usable = [m for m in manifest if not m["on_mosaic_edge"]]
    print(f"\n  ✓ {len(manifest):,} dossiers rendered and hashed ({len(usable):,} away from mosaic edges)")
    print(f"  ✓ hash verification: {verify['checked']:,} checked, {verify['hash_mismatches']} mismatches")
    print(f"  ✓ QA: {qa['checked']} random dossiers spot-checked, {qa['with_problems']} with problems")
    assert len(manifest) >= 4_990
    assert verify["hash_mismatches"] == 0 and verify["checked"] >= len(manifest) * 0.01
    assert qa["checked"] >= 40 and qa["with_problems"] == 0
    assert all(len(m["dossier_sha256"]) == 64 for m in manifest)


def test_t1_7_rgb_common_grid_aligns_mixed_pixel_scales():
    import numpy as np
    from sc_curation import dossiers

    n = dossiers.SIZE
    yy, xx = np.mgrid[0:n, 0:n]
    c = n // 2
    fine = np.exp(-((xx - c) ** 2 + (yy - c) ** 2) / (2 * 6.0**2)).astype(np.float32)
    coarse = np.exp(-((xx - c) ** 2 + (yy - c) ** 2) / (2 * 3.0**2)).astype(np.float32) * 4
    grid, fov = dossiers.common_grid({"f150w": fine, "f444w": coarse}, {"f150w": 0.02, "f444w": 0.04})
    assert fov == round(n * 0.02, 2)
    assert grid["f444w"].shape == grid["f150w"].shape == (n, n)
    peak = lambda a: np.unravel_index(np.nanargmax(a), a.shape)
    assert np.abs(np.subtract(peak(grid["f444w"]), peak(grid["f150w"]))).max() <= 2
    assert np.isclose(np.nanmax(grid["f444w"]), np.nanmax(grid["f150w"]), rtol=0.05)
    same, fov2 = dossiers.common_grid({"f150w": coarse}, {"f150w": 0.04})
    assert fov2 == round(n * 0.04, 2) and np.array_equal(same["f150w"], coarse)
