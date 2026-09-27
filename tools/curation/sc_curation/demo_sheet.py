import argparse
import io
import json
import time
import urllib.request
from pathlib import Path

from PIL import Image, ImageDraw

THUMB = "https://grizli-cutout.herokuapp.com/thumb?coord={ra},{dec}&size=3&filters=f150w-clear,f277w-clear,f444w-clear&asinh=True"
CACHE = Path.home() / ".cache" / "space-compute" / "thumbs"


def thumb(ra: float, dec: float) -> Image.Image:
    CACHE.mkdir(parents=True, exist_ok=True)
    path = CACHE / f"{ra:.6f}_{dec:.6f}.png"
    if not path.exists():
        with urllib.request.urlopen(THUMB.format(ra=ra, dec=dec), timeout=60) as r:
            path.write_bytes(r.read())
        time.sleep(0.5)
    return Image.open(io.BytesIO(path.read_bytes())).convert("RGB")


def build(sel_dir: Path, out: Path, per_group: int = 8, tile: int = 150) -> Path:
    rows = {json.loads(l)["subject_id"]: json.loads(l) for l in (sel_dir / "selection_v1.jsonl").read_text().splitlines()}
    gold = json.loads((sel_dir / "gold_v1.json").read_text())
    groups = {}
    for sid, g in gold.items():
        a = g["answers"]
        label = "weak: no merger" if g.get("strength") == "weak" else "star/artifact" if a.get("shape") == "artifact" else (
            "spiral" if a.get("spiral") == "yes" else "edge-on" if a.get("edgeon") == "yes" else
            "merger" if a.get("merger") == "major" else a.get("shape", "featured"))
        if int(sid) in rows:
            groups.setdefault(f"gold: {label}", []).append((int(sid), ", ".join(f"{k}={v}" for k, v in sorted(a.items()))))
    reds = [(sid, f"z={r['z_phot']}") for sid, r in rows.items() if r["reason"] == "red_compact"]
    groups["red compact (LRD candidates)"] = reds
    picked = [(name, items[:per_group]) for name, items in sorted(groups.items()) if items]
    width = per_group * tile
    height = sum(tile + 44 for _ in picked)
    sheet = Image.new("RGB", (width, height), (8, 10, 20))
    draw = ImageDraw.Draw(sheet)
    y = 0
    for name, items in picked:
        draw.text((6, y + 4), f"{name}  ({len(groups[name])} in selection)", fill=(242, 184, 75))
        for k, (sid, note) in enumerate(items):
            r = rows[sid]
            img = thumb(r["ra_deg"], r["dec_deg"]).resize((tile, tile))
            sheet.paste(img, (k * tile, y + 20))
            draw.text((k * tile + 3, y + 22 + tile), f"{r['field']} {sid}"[:24], fill="white")
        y += tile + 44
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    return out


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("sel_dir")
    p.add_argument("out")
    a = p.parse_args()
    print(build(Path(a.sel_dir), Path(a.out)))


if __name__ == "__main__":
    main()
