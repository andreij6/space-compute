import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw


def build(root: Path, out: Path, columns: int = 10, tile: int = 128, limit: int = 50) -> Path:
    rows = [json.loads(line) for line in (root / "manifest_v1.jsonl").read_text().splitlines()][:limit]
    lines = (len(rows) + columns - 1) // columns
    sheet = Image.new("RGB", (columns * tile, lines * (tile + 16)), "black")
    draw = ImageDraw.Draw(sheet)
    for i, row in enumerate(rows):
        img = Image.open(root / row["rgb_url"]).convert("RGB").resize((tile, tile), Image.NEAREST)
        x, y = (i % columns) * tile, (i // columns) * (tile + 16)
        sheet.paste(img, (x, y))
        label = f"{row['subject_id']} {row.get('kind', row['field'])}"
        draw.text((x + 3, y + tile + 2), label[:22], fill="white")
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    return out


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("root")
    p.add_argument("out")
    p.add_argument("--limit", type=int, default=50)
    a = p.parse_args()
    print(build(Path(a.root), Path(a.out), limit=a.limit))


if __name__ == "__main__":
    main()
