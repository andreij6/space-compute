import os
from pathlib import Path

DJA = "https://s3.amazonaws.com/grizli-v2/JwstMosaics/v7/"
CACHE = Path(os.environ.get("SC_CACHE", Path.home() / ".cache" / "space-compute" / "dja"))

FIELDS = {
    "ceers": [("ceers-full-grizli-v7.4", "tar")],
    "jades-gds": [("gds-grizli-v7.2", "tar")],
    "jades-gdn": [("gdn-grizli-v7.3", "zout")],
    "primer-uds": [("primer-uds-north-grizli-v7.2", "tar"), ("primer-uds-south-grizli-v7.2", "tar")],
    "primer-cosmos": [("primer-cosmos-east-grizli-v7.0", "tar"), ("primer-cosmos-west-grizli-v7.0", "tar")],
    "abell2744": [("abell2744clu-grizli-v7.2", "zout")],
}


def phot_name(root: str) -> str:
    return f"{root}-fix_phot.fits"


def zout_name(root: str) -> str:
    return f"{root}-fix.eazypy.zout.fits"


def photoz_tar_name(root: str) -> str:
    return f"{root}-fix.photoz.tar.gz"
