import argparse
import sys
from pathlib import Path

import numpy as np
from astropy.io import fits


def load(path):
    with fits.open(path) as hdul:
        for hdu in hdul:
            if hdu.data is not None:
                return np.nan_to_num(np.asarray(hdu.data, dtype=float))
    raise ValueError(f"{path}: no image data")


def aperture_flux(img, radius_px):
    cy, cx = (np.array(img.shape) - 1) / 2
    y, x = np.indices(img.shape)
    return float(img[(y - cy) ** 2 + (x - cx) ** 2 <= radius_px**2].sum())


def color(blue, red, radius_px=10):
    fb, fr = aperture_flux(blue, radius_px), aperture_flux(red, radius_px)
    if fb <= 0 or fr <= 0:
        return None
    return float(-2.5 * np.log10(fb / fr))


def residual(a, b, radius_px=10):
    fa, fb = aperture_flux(a, radius_px), aperture_flux(b, radius_px)
    if fa <= 0 or fb <= 0:
        raise ValueError("non-positive central flux")
    return a / fa - b / fb


def main(argv=None):
    p = argparse.ArgumentParser(description="Colour and residual checks on per-filter FITS cutouts")
    p.add_argument("blue", type=Path)
    p.add_argument("red", type=Path)
    p.add_argument("--radius-px", type=float, default=10)
    p.add_argument("--residual-out", type=Path)
    a = p.parse_args(argv)
    blue, red = load(a.blue), load(a.red)
    c = color(blue, red, a.radius_px)
    print(f"{a.blue.stem}-{a.red.stem} = {'n/a' if c is None else f'{c:.2f} mag'} (r={a.radius_px}px)")
    if a.residual_out:
        r = residual(blue, red, a.radius_px)
        fits.PrimaryHDU(r).writeto(a.residual_out, overwrite=True)
        print(f"residual -> {a.residual_out} (min {r.min():.3g}, max {r.max():.3g})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
