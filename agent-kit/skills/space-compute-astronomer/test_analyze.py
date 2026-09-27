import numpy as np
from astropy.io import fits

import analyze


def gauss(amp, sigma, n=41):
    y, x = np.indices((n, n)) - (n - 1) / 2
    return amp * np.exp(-(x**2 + y**2) / (2 * sigma**2))


def test_color_is_minus_2p5_log_flux_ratio(tmp_path):
    fits.PrimaryHDU(gauss(1.0, 3)).writeto(tmp_path / "f277w.fits")
    fits.PrimaryHDU(gauss(10.0, 3)).writeto(tmp_path / "f444w.fits")
    c = analyze.color(analyze.load(tmp_path / "f277w.fits"), analyze.load(tmp_path / "f444w.fits"))
    assert abs(c - 2.5) < 1e-9


def test_color_none_for_empty_band():
    assert analyze.color(np.zeros((21, 21)), gauss(1.0, 3, 21)) is None


def test_residual_zero_for_same_shape_and_positive_for_extra_arc():
    base = gauss(5.0, 3)
    assert np.allclose(analyze.residual(base, 2 * base), 0)
    arc = base.copy()
    arc[5, 10:30] += 3.0
    r = analyze.residual(arc, base, radius_px=25)
    assert r[5, 10:30].min() > 0


def test_cli_writes_residual(tmp_path, capsys):
    fits.PrimaryHDU(gauss(2.0, 3)).writeto(tmp_path / "b.fits")
    fits.PrimaryHDU(gauss(1.0, 3)).writeto(tmp_path / "r.fits")
    assert analyze.main([str(tmp_path / "b.fits"), str(tmp_path / "r.fits"), "--residual-out", str(tmp_path / "res.fits")]) == 0
    assert "b-r = -0.75 mag" in capsys.readouterr().out
    assert (tmp_path / "res.fits").exists()
