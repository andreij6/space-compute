import hashlib
import io

import numpy as np
from astropy.io import fits
from PIL import Image

STRETCH = {"type": "asinh", "a": 0.1, "min": -0.02, "max": 1.2}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def asinh_rgb(r: np.ndarray, g: np.ndarray, b: np.ndarray, stretch: dict = STRETCH) -> bytes:
    lo, hi, a = stretch["min"], stretch["max"], stretch["a"]
    channels = []
    for band in (r, g, b):
        x = np.clip((band - lo) / (hi - lo), 0, 1)
        y = np.arcsinh(x / a) / np.arcsinh(1 / a)
        channels.append(np.round(y * 255).astype(np.uint8))
    img = Image.fromarray(np.flipud(np.dstack(channels)), mode="RGB")
    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=False, compress_level=6)
    return buf.getvalue()


def fits_bytes(data: np.ndarray, header: dict, weight: np.ndarray | None = None) -> bytes:
    hdr = fits.Header()
    for k, v in header.items():
        hdr[k] = v
    hdus = [fits.PrimaryHDU(data=data.astype(np.float32), header=hdr)]
    if weight is not None:
        hdus.append(fits.ImageHDU(data=weight.astype(np.float32), name="WHT"))
    buf = io.BytesIO()
    fits.HDUList(hdus).writeto(buf, checksum=False)
    return buf.getvalue()
