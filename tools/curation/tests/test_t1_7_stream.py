import gzip

import numpy as np
from astropy.io import fits
from astropy.wcs import WCS

from sc_curation.stream import cut_mosaic


def make_mosaic(path, nx=300, ny=200):
    w = WCS(naxis=2)
    w.wcs.ctype = ["RA---TAN", "DEC--TAN"]
    w.wcs.crval = [150.0, 2.0]
    w.wcs.crpix = [nx / 2, ny / 2]
    w.wcs.cd = [[-1e-5, 0], [0, 1e-5]]
    data = (np.arange(ny)[:, None] * 1000 + np.arange(nx)[None, :]).astype(np.float32)
    hdu = fits.PrimaryHDU(data, header=w.to_header())
    raw = path.with_suffix("")
    hdu.writeto(raw)
    path.write_bytes(gzip.compress(raw.read_bytes()))
    return data, w


def test_t1_7_streamed_cutouts_equal_direct_slices_including_edges(tmp_path):
    src = tmp_path / "mosaic.fits.gz"
    data, w = make_mosaic(src)
    pix = np.array([[150.0, 100.0], [5.0, 5.0], [295.0, 190.0], [60.0, 150.0]])
    ra, dec = w.all_pix2world(pix[:, 0], pix[:, 1], 0)
    size = 21
    header, corners = cut_mosaic(src.as_uri(), ra, dec, size, tmp_path / "cube.npy")
    cube = np.load(tmp_path / "cube.npy")
    assert header["NAXIS1"] == 300
    for k, (x0, y0) in enumerate(corners):
        for j in range(size):
            for i in range(size):
                y, x = y0 + j, x0 + i
                expected = data[y, x] if 0 <= y < 200 and 0 <= x < 300 else np.nan
                got = cube[k, j, i]
                assert (np.isnan(expected) and np.isnan(got)) or got == expected
    centre = cube[0, size // 2, size // 2]
    assert centre == data[100, 150]
