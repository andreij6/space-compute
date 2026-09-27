import argparse
import io
import json
import random
from pathlib import Path

import numpy as np
from astropy.io import fits
from astropy.wcs import WCS
from PIL import Image, ImageDraw


def check(folder: Path) -> list[str]:
    dossier = json.loads((folder / "dossier.json").read_text())
    problems = []
    t = dossier["target"]
    bands = [f["filter"] for f in dossier["images"]["fits"]]
    if len(bands) < 5:
        problems.append(f"only {len(bands)} bands")
    for f in dossier["images"]["fits"]:
        with fits.open(io.BytesIO((folder / f["url"]).read_bytes())) as hdul:
            hdu = hdul[1]
            data = hdu.data
            w = WCS(hdu.header)
            ny, nx = data.shape
            ra, dec = w.all_pix2world((nx - 1) / 2, (ny - 1) / 2, 0)
            off = np.hypot((ra - t["ra_deg"]) * np.cos(np.radians(t["dec_deg"])), dec - t["dec_deg"]) * 3600
            if off > 0.1:
                problems.append(f"{f['filter']} centre off by {off:.2f}\"")
            if not np.any(data != 0):
                problems.append(f"{f['filter']} empty")
    return problems


def sheet(bucket: Path, ids: list[int], out: Path, tile: int = 192) -> None:
    cols = 8
    rows = (len(ids) + cols - 1) // cols
    img = Image.new("RGB", (cols * tile, rows * (tile + 30)), (8, 10, 20))
    draw = ImageDraw.Draw(img)
    for k, sid in enumerate(ids):
        folder = bucket / "v1" / "subjects" / str(sid)
        d = json.loads((folder / "dossier.json").read_text())
        x, y = (k % cols) * tile, (k // cols) * (tile + 30)
        img.paste(Image.open(folder / "rgb.png").convert("RGB").resize((tile, tile)), (x, y))
        z = d["redshift"]["z_phot"]
        draw.text((x + 3, y + tile + 2), f"{d['target']['field']} {sid}", fill="white")
        draw.text((x + 3, y + tile + 15), f"z={z}  mag={d['photometry']['bands']['f444w']['mag_ab']}", fill=(170, 180, 200))
    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)


def main() -> None:
    ap = argparse.ArgumentParser(description="Automated + visual QA of rendered dossiers")
    ap.add_argument("--bucket", default="../../target/bucket")
    ap.add_argument("--n", type=int, default=40)
    ap.add_argument("--out", default="../../docs/demos/T1.7/qa-40.png")
    ap.add_argument("--seed", type=int, default=7)
    a = ap.parse_args()
    bucket = Path(a.bucket)
    rows = [json.loads(l) for l in (bucket / "manifest_v1.jsonl").read_text().splitlines()]
    picks = random.Random(a.seed).sample(rows, min(a.n, len(rows)))
    report = {}
    for r in picks:
        problems = check(bucket / Path(r["dossier_url"]).parent)
        report[r["subject_id"]] = problems
    sheet(bucket, [r["subject_id"] for r in picks], Path(a.out))
    bad = {k: v for k, v in report.items() if v}
    print(json.dumps({"checked": len(report), "with_problems": len(bad), "problems": bad}, indent=1))


if __name__ == "__main__":
    main()
