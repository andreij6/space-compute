import subprocess
import sys
import tarfile

from .sources import CACHE, DJA, FIELDS, photoz_tar_name, phot_name, zout_name


def download(name: str) -> None:
    dest = CACHE / name
    if dest.exists() and dest.stat().st_size > 0 and not (CACHE / f"{name}.part").exists():
        return
    CACHE.mkdir(parents=True, exist_ok=True)
    part = CACHE / f"{name}.part"
    subprocess.run(["curl", "-sSfL", "-C", "-", "-o", str(part), DJA + name], check=True)
    part.rename(dest)


def extract_zout(root: str) -> None:
    target = CACHE / zout_name(root)
    if target.exists():
        return
    with tarfile.open(CACHE / photoz_tar_name(root), "r:gz") as tar:
        for member in tar:
            if member.name.endswith(".zout.fits"):
                with tar.extractfile(member) as src:
                    target.write_bytes(src.read())
                return
    raise SystemExit(f"no zout.fits inside {photoz_tar_name(root)}")


def main() -> None:
    for field, roots in FIELDS.items():
        for root, kind in roots:
            download(phot_name(root))
            if kind == "zout":
                download(zout_name(root))
            else:
                download(photoz_tar_name(root))
                extract_zout(root)
            print(f"ready {field}: {root}", flush=True)


if __name__ == "__main__":
    sys.exit(main())
