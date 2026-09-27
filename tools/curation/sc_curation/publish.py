import argparse
import hashlib
import json
import os
import random
import subprocess
import sys
from pathlib import Path

R2_ENV = ("R2_ACCOUNT_ID", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY", "R2_BUCKET")


def verify(bucket: Path, sample: float = 1.0, seed: int = 1) -> dict:
    rows = [json.loads(l) for l in (bucket / "manifest_v1.jsonl").read_text().splitlines()]
    rng = random.Random(seed)
    checked = bad = 0
    for row in rows:
        if sample < 1.0 and rng.random() > sample:
            continue
        checked += 1
        body = (bucket / row["dossier_url"]).read_bytes()
        ok = hashlib.sha256(body).hexdigest() == row["dossier_sha256"]
        ok &= hashlib.sha256((bucket / row["rgb_url"]).read_bytes()).hexdigest() == row["rgb_sha256"]
        dossier = json.loads(body)
        folder = (bucket / row["dossier_url"]).parent
        for f in [dossier["images"]["rgb_sw"], dossier["images"]["segmentation"], *dossier["images"]["fits"]]:
            ok &= hashlib.sha256((folder / f["url"]).read_bytes()).hexdigest() == f["sha256"]
        bad += 0 if ok else 1
    return {"dossiers": len(rows), "checked": checked, "hash_mismatches": bad}


def upload(bucket: Path) -> None:
    missing = [k for k in R2_ENV if not os.environ.get(k)]
    if missing:
        raise SystemExit(f"R2 upload needs {', '.join(missing)} (owner task T8.14); local bucket at {bucket} is ready")
    endpoint = f"https://{os.environ['R2_ACCOUNT_ID']}.r2.cloudflarestorage.com"
    env = {**os.environ, "AWS_ACCESS_KEY_ID": os.environ["R2_ACCESS_KEY_ID"],
           "AWS_SECRET_ACCESS_KEY": os.environ["R2_SECRET_ACCESS_KEY"], "AWS_DEFAULT_REGION": "auto"}
    target = f"s3://{os.environ['R2_BUCKET']}/"
    subprocess.run(["aws", "s3", "sync", str(bucket), target, "--endpoint-url", endpoint, "--size-only",
                    "--exclude", "manifest_v1.*.jsonl"], check=True, env=env)


def main() -> None:
    ap = argparse.ArgumentParser(description="Verify the local bucket and publish it to R2")
    ap.add_argument("--bucket", default="../../target/bucket")
    ap.add_argument("--sample", type=float, default=1.0)
    ap.add_argument("--upload", action="store_true")
    a = ap.parse_args()
    report = verify(Path(a.bucket), a.sample)
    print(json.dumps(report))
    if report["hash_mismatches"]:
        sys.exit(1)
    if a.upload:
        upload(Path(a.bucket))


if __name__ == "__main__":
    main()
