# Peer Review: JWST Subject Data Architecture & Ingestion Suggestions

**Target Specifications:** [`docs/specs/07-data-curation.md`](../docs/specs/07-data-curation.md), [`docs/specs/05-frontend.md`](../docs/specs/05-frontend.md), [`docs/specs/01-architecture.md`](../docs/specs/01-architecture.md)  
**Date:** September 2026  
**Status:** Recommendations for Implementation  

---

## Executive Summary

Space Compute's approach to JWST subject curation—pre-rendering static cutouts and structured dossiers to Cloudflare R2 while recording cryptographic hashes on the Internet Computer—is an exemplary architectural pattern. It decouples high-frequency agent activity from scientific archive servers, guarantees immutable provenance, and eliminates egress costs.

However, an audit of the astronomical data standards, STScI/MAST terms of use, and DAWN JWST Archive (DJA) specifications identified several critical nuances. This document details actionable suggestions to guarantee scientific accuracy, legal compliance, and long-term research credibility.

---

## 1. Photometric Calibration & FITS Header Standardization (High Priority)

### The Issue
In `07-data-curation.md §3`, the per-filter science cutout (`<filter>.fits`) specifies:
> `Full WCS header, PHOTFNU, PIXAR_SR, units, plus a weight extension.`

There is an important discrepancy between native JWST pipeline mosaics and DJA mosaics:
* **Standard STScI Pipeline:** Resampled mosaics (`_i2d.fits`) express pixel values in surface brightness units of **$\text{MJy/sr}$**. Flux conversion requires multiplying by the pixel solid angle (`PIXAR_SR` in steradians) and scaling to $\mu\text{Jy}$ using `PHOTFNU` or unit conversions.
* **DAWN JWST Archive (DJA / `grizli` v7):** DJA mosaics (`_drz_sci.fits`) are normalized to **$10\ \text{nJy/pixel}$**, corresponding to a uniform AB magnitude zeropoint of **$28.9$**.

### Risk
If an autonomous agent (or an operator script analyzing the FITS cutouts) assumes standard STScI $\text{MJy/sr}$ surface brightness conventions rather than DJA's $10\ \text{nJy/pixel}$ scale, calculated flux densities and AB magnitudes will be off by orders of magnitude.

### Recommendations
1. **Explicit FITS Header Injection in `render.py`:**
   When extracting cutouts with `astropy.nddata.Cutout2D`, write standardized calibration keywords into the primary FITS header:
   ```fits
   BUNIT   = '10 nJy/pixel'           / Physical units of pixel values
   PHOTZERO=                 28.9     / AB magnitude zero point
   PHOTSYS = 'AB'                     / Photometric system
   FLUXUNIT= 'nJy'                    / Unit for conversion
   SCALEFAC=                 10.0     / Scale factor: 1 pixel value = 10 nJy
   ```
2. **Explicit Metadata Block in `dossier.json`:**
   Under `"images"."fits"`, document the exact calibration parameters alongside pivot wavelengths:
   ```json
   "fits_calibration": {
     "pixel_intensity_unit": "10 nJy/pixel",
     "zeropoint_ab": 28.9,
     "conversion_to_microjansky": 0.01,
     "bunit": "10 nJy/pixel"
   }
   ```

---

## 2. Formal Legal Attribution & Grant Compliance (Medium Priority)

### The Issue
NASA, ESA, STScI, and the Cosmic Dawn Center require specific attribution language for scientific publications and derivative open data products. While `07-data-curation.md` includes the general STScI contract acknowledgment, three mandatory attribution requirements are currently missing or incomplete:

1. **STScI Zero-Exclusive-Access Clause:** Required for observations originating from Early Release Science (ERS), Director's Discretionary (DD), or Large/Treasury programs where the PI waived proprietary data rights.
2. **DJA Grant Attribution:** Required by the Cosmic Dawn Center for using DJA data products.
3. **Software & Pipeline DOIs:** Recommended for reproducibility tracking.

### Recommendations
Update the `provenance` object in `dossier.json` schema (`sc-dossier/1`), the `/credits` screen, and the public citation footer:

```json
"provenance": {
  "programs": [
    { "pid": 1345, "name": "CEERS", "pi": "Finkelstein" }
  ],
  "credits": [
    "Dawn JWST Archive (Valentino et al. 2023; grizli, Brammer)",
    "CEERS (Finkelstein et al.)"
  ],
  "acknowledgments": {
    "stsci": "This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope, obtained from the Mikulski Archive for Space Telescopes at STScI, operated by AURA under NASA contract NAS5-03127 for JWST.",
    "zero_exclusive_access": "The authors acknowledge the CEERS, JADES, UNCOVER, and PRIMER observing teams for developing their programs with a zero-exclusive-access period.",
    "dja": "The data products presented herein were retrieved from the DAWN JWST Archive (DJA). DJA is an initiative of the Cosmic Dawn Center (DAWN), which is funded by the Danish National Research Foundation under grant DNRF140."
  },
  "dois": {
    "dja_reduction": "10.3847/1538-4365/acb5ed",
    "grizli": "10.5281/zenodo.1146904",
    "msaexp": "10.5281/zenodo.7299500"
  }
}
```

---

## 3. External Archive Cross-Referencing & Verification Links (High Value)

### The Opportunity
When an AI agent flags an anomalous object (e.g. `lensed_arc`, `little_red_dot`, or `high_z_candidate`), human researchers and peer reviewers on the web interface will want to cross-reference the raw archival exposures or inspect the broader cosmological environment.

### Recommendations
1. **Include External Direct URLs in `dossier.json`:**
   Add pre-computed lookup URLs into `target`:
   * **MAST Portal Search URL:** Direct RA/Dec cone-search link.
   * **DJA Viewer URL:** Direct link to the coordinates in the DJA interactive web interface.
   * **ADS / NASA Bibcode Query:** Pre-formatted search link for survey papers covering the coordinates.
2. **Display "Verify in Archive" Buttons on `/d/:publicId`:**
   In the Scientific Dossier sidebar of the Discovery Detail screen, render external verification buttons:
   * `[Open in MAST]`
   * `[Open in DJA]`
   * `[Download Raw FITS Cutout]`

---

## 4. Ground Truth & Galaxy Zoo CEERS Strategy (Risk Mitigation)

### Context & Risk (Spike SP-7)
The Galaxy Zoo CEERS dataset is designated as the primary source for visual morphology gold labels. However, the formal Galaxy Zoo CEERS paper and public data release are actively in progress. Gating reputation calibration or system deployment on an external third-party release schedule introduces timeline risk.

### Recommendations
1. **Strengthen the Objective Gold Pipeline (`gold.py`):**
   Prioritize the objective gold fallback defined in `07-data-curation.md §5`:
   * **Stars & Artifacts:** Flagged with 100% confidence from catalog stellarity flags (`star_flag`, $\chi^2$ morphology, diffraction spikes).
   * **Spectroscopically Confirmed High-$z$ Objects:** From DJA NIRSpec extractions (`msaexp` quality grade $\ge 3$) to evaluate `high_z_candidate` and dropout classifications.
   * **Confirmed Strong Lenses:** Using the published UNCOVER (Abell 2744) cluster lens models and verified multiply-imaged systems.
2. **Independent Gold Subsets:**
   Ensure the 2,000 gold targets are decoupled from subjective morphology voting so that basic agent competence (shape, artifact detection, star recognition) can be scored deterministically on day one.

---

## 5. Noise & Weight Map Handling in `<filter>.fits`

### The Issue
Astrophysical significance tests (e.g., assessing whether a clump is a true star-forming knot or a noise fluctuation) require pixel-level inverse variance or weight maps.

### Recommendations
1. **Multi-Extension Cutout Format:**
   Ensure each `<filter>.fits` file follows standard Multi-Extension FITS (MEF) conventions:
   * **Extension 0 (Primary / `SCI`):** Science cutout flux values ($10\ \text{nJy/pixel}$).
   * **Extension 1 (`WHT`):** Inverse-variance weight map extracted from the corresponding `*_drz_wht.fits` mosaic.
2. **Error Calculation Guidance in `protocol_v1`:**
   Provide explicit documentation in the agent toolkit (`agent-kit`) explaining how agents should calculate pixel noise:
   $$\sigma_i = \frac{1}{\sqrt{\text{WHT}_i}}$$

---

## 6. Implementation Checklist & Task Mapping

| Task ID | Component / File | Suggested Action | Status |
| :--- | :--- | :--- | :---: |
| **FE-0 / T6.3** | `docs/specs/05-frontend.md` | Add DJA grant `DNRF140` and zero-exclusive-access acknowledgments to `/credits`. | Pending |
| **T1.5** | `tools/curation/render.py` | Inject `BUNIT = '10 nJy/pixel'` and `PHOTZERO = 28.9` into cutout FITS headers; package `SCI` and `WHT` extensions. | Pending |
| **T1.5** | `tools/curation/dossier.py` | Add `fits_calibration`, `acknowledgments`, `dois`, and external archive links to `dossier.json`. | Pending |
| **T1.5** | `tools/curation/gold.py` | Build objective gold criteria (stars, spec-$z$, lenses) independently of GZ CEERS release status. | Pending |
| **T6.3** | `frontend/src/screens/DiscoveryDetail.tsx` | Add external "Verify in MAST" and "Verify in DJA" action buttons in the dossier drawer. | Pending |
