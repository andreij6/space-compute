import subprocess
import zlib
from pathlib import Path

import numpy as np
from astropy.io import fits
from astropy.wcs import WCS

BLOCK = 2880


class GzipStream:
    def __init__(self, url: str):
        self.proc = subprocess.Popen(["curl", "-sSfL", url], stdout=subprocess.PIPE, bufsize=1 << 20)
        self.inflate = zlib.decompressobj(16 + zlib.MAX_WBITS)
        self.buf = bytearray()

    def read(self, n: int) -> bytes:
        while len(self.buf) < n:
            chunk = self.proc.stdout.read(1 << 20)
            if not chunk:
                self.buf += self.inflate.flush()
                break
            self.buf += self.inflate.decompress(chunk)
        out = bytes(self.buf[:n])
        del self.buf[:n]
        return out

    def close(self) -> None:
        self.proc.stdout.close()
        self.proc.kill()
        self.proc.wait()


def read_header(stream) -> fits.Header:
    raw = b""
    while True:
        block = stream.read(BLOCK)
        if len(block) < BLOCK:
            raise ValueError("truncated FITS header")
        raw += block
        if any(block[i:i + 8] == b"END     " for i in range(0, BLOCK, 80)):
            return fits.Header.fromstring(raw.decode("ascii"))


def cut_mosaic(url: str, ra: np.ndarray, dec: np.ndarray, size: int, out: Path) -> tuple[fits.Header, np.ndarray]:
    stream = GzipStream(url)
    try:
        header = read_header(stream)
        wcs = WCS(header)
        nx, ny, bitpix = header["NAXIS1"], header["NAXIS2"], header["BITPIX"]
        dtype = {-32: ">f4", 32: ">i4", 16: ">i2", -64: ">f8"}[bitpix]
        x, y = wcs.all_world2pix(ra, dec, 0)
        x0 = np.round(x).astype(int) - size // 2
        y0 = np.round(y).astype(int) - size // 2
        cube = np.lib.format.open_memmap(out, mode="w+", dtype=np.float32, shape=(len(ra), size, size))
        cube[:] = np.nan
        order = np.argsort(y0)
        y0s, starts = y0[order], 0
        active = np.array([], dtype=int)
        cols = np.arange(size)
        row_bytes = nx * abs(bitpix) // 8
        last_needed = int(y0.max() + size) if len(y0) else -1
        for row in range(min(ny, max(last_needed, 0))):
            data = np.frombuffer(stream.read(row_bytes), dtype=dtype)
            while starts < len(order) and y0s[starts] <= row:
                active = np.append(active, order[starts])
                starts += 1
            if len(active) == 0:
                continue
            active = active[(y0[active] + size) > row]
            if len(active) == 0:
                continue
            xs = x0[active][:, None] + cols[None, :]
            valid = (xs >= 0) & (xs < nx)
            vals = np.where(valid, data[np.clip(xs, 0, nx - 1)], np.nan).astype(np.float32)
            cube[active, row - y0[active], :] = vals
        cube.flush()
        return header, np.stack([x0, y0], axis=1)
    finally:
        stream.close()
