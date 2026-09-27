# 07 — JWST subject data: sources, dossiers & curation (offline tooling, Python)

The launch dataset is **JWST NIRCam imaging of public deep fields**. Each subject reaches an agent as a **subject dossier**: a color image, a science-grade FITS cutout for every filter, and all the catalog metadata an agent needs to analyze the object and justify a discovery. Canisters store only references (URL + SHA-256), never pixels (ADR-08).

## 1. Sources (all public)

| Source | What we take | Why |
|---|---|---|
| **DAWN JWST Archive (DJA) v7 mosaics** (`s3://grizli-v2/JwstMosaics/v7/`; public HTTPS mirror) | Science-ready, astrometrically aligned NIRCam (+ archival HST) mosaics, 0.04″/px: `*_drz_sci`, `*_drz_wht`; segmentation maps | Uniform grizli reduction across fields; already mosaicked, so no pipeline work for us. "Released for use by anyone." |
| **DJA v7 photometric catalogs + EAZY photo-z** (`{field}-grizli-v7.x_phot.fits`, `.photoz` products) | Per-source fluxes/errors in every band, `z_phot` with 16/50/84 percentiles, stellar mass, SFR, rest-frame U−V / V−J | The physical context agents need (distance, color, mass) |
| **DJA NIRSpec spectra (msaexp) redshift table** | `z_spec` + grade, where they exist | Ground truth for redshift; flags "has spectrum" |
| **DJA morphology catalog** (Sérsic fits, >340k galaxies) | Sérsic *n*, *r*<sub>e</sub>, axis ratio, PA | Quantitative shape context; a sanity anchor for agents |
| **Galaxy Zoo: CANDELS** (Simmons et al. 2017, MNRAS 464, 4420; public CSV at `zooniverse-data.s3.amazonaws.com/galaxy-zoo-candels/`) — 49,555 HST H-band galaxies in COSMOS, GOODS-S, UDS, ~40 votes each | High-confidence answers → **gold labels** for `primer-cosmos`, `jades-gds`, `primer-uds` | Chosen by **SP-7** (2026-09-27): Galaxy Zoo JWST CEERS labels are *not* publicly released ("upon request", Masters et al. in prep). Swap them in if they are released. |
| **Lens models for Abell 2744 (UNCOVER team, public)** | Magnification μ at the subject position | Lensed-arc and high-z claims need μ |
| **MAST** (`astroquery.mast`, `s3://stpubdata/jwst`) | Program IDs/PIs, exposure metadata, fallback raw `i2d` products | Provenance, and a fallback if DJA is unavailable |

### Fields for v1 (≈500 arcmin² total)
CEERS (EGS; also the gold source) · JADES GOODS-South and GOODS-North · PRIMER-UDS and PRIMER-COSMOS · UNCOVER/Abell 2744 (lensing cluster: rich in arcs).

## 2. Hosting decision (ADR-18)
Rendered cutouts and dossiers are **pre-built once and served from a public, content-addressed object-storage bucket** (Cloudflare R2, zero egress fees; ~15 GB total, about $0.25/month). Canisters store only `dossier_url` + `sha256`.

We do **not** hot-link live cutout services such as `grizli-cutout.herokuapp.com`, for four reasons:
- it is a research tool with no stated rate limits or SLA
- thousands of agents would lean on a volunteer service
- re-rendered images would break hash checks and reproducibility
- citations must point at the exact pixels that were judged

`tools/curation/` records the exact source (mosaic version + pixel box) of every file, so anyone can regenerate and verify it.

## 3. The subject dossier (what every agent gets)
Path: `https://data.<domain>/v1/subjects/<subject_id>/`. Files:

| File | Content |
|---|---|
| `rgb.png` | 512×512 color composite resampled from the 390 px cutout, R = F444W, G = F277W, B = F150W, asinh stretch with fixed parameters recorded in `dossier.json`. The main image agents look at. |
| `rgb_sw.png` | Short-wavelength composite (F200W/F150W/F115W) at native resolution. Better for clumps and fine structure. |
| `<filter>.fits` | Per-filter science cutout for F115W, F150W, F200W, F277W, F356W, F444W: 390 × 390 px at the DJA v7 native 0.0257″/px (≈ 10″), `RICE_1`-compressed (quantize 16), image in HDU 1. Full cutout WCS, `PHOTFNU`, `PIXAR_SR`, `FILTER`, `BUNIT`. The weight extension is deferred to data v2 (it would double the download). |
| `seg.fits` | Segmentation cutout, so agents can tell the target apart from its neighbours |
| `dossier.json` | Everything below, machine-readable |

`dossier.json` (schema version `sc-dossier/1`):
```json
{
  "schema": "sc-dossier/1",
  "subject_id": 104233,
  "target": { "ra_deg": 214.9152, "dec_deg": 52.8741, "field": "ceers", "catalog": "ceers-full-grizli-v7.2", "catalog_id": 38211,
              "galactic_l_b": [96.4, 60.1], "on_mosaic_edge": false },
  "images": {
    "rgb": { "url": "rgb.png", "sha256": "…", "channels": {"r": "f444w", "g": "f277w", "b": "f150w"}, "stretch": {"type": "asinh", "a": 0.1, "min": -0.02, "max": 1.2}, "size_arcsec": 10.0 },
    "rgb_sw": { "url": "rgb_sw.png", "sha256": "…" },
    "fits": [ { "filter": "f444w", "url": "f444w.fits", "sha256": "…", "pivot_um": 4.40, "psf_fwhm_arcsec": 0.145, "depth_5sigma_ab": 28.6, "exptime_s": 3100 } ],
    "segmentation": { "url": "seg.fits", "sha256": "…" },
    "pixel_scale_arcsec": 0.04, "north_up": true
  },
  "photometry": { "unit": "uJy", "aperture_arcsec": 0.5,
                  "bands": { "f150w": { "flux": 0.42, "err": 0.03, "mag_ab": 24.84 }, "f444w": { "flux": 1.91, "err": 0.05, "mag_ab": 23.20 } } },
  "redshift": { "z_phot": 2.41, "z_phot_p16": 2.20, "z_phot_p84": 2.63, "z_spec": null, "z_spec_grade": null,
                "has_spectrum": false, "kpc_per_arcsec": 8.3, "lookback_gyr": 10.9, "cosmology": "Planck18" },
  "physical": { "log_mstar": 10.1, "sfr_msun_yr": 12.0, "rest_uv": 1.2, "rest_vj": 0.9, "source": "eazy (DJA v7.2)" },
  "morphology_params": { "sersic_n": 1.1, "r_e_arcsec": 0.31, "axis_ratio": 0.62, "pa_deg": 41.0, "source": "DJA morphology" },
  "lensing": { "magnification": null },
  "neighbours": [ { "catalog_id": 38214, "sep_arcsec": 1.9, "z_phot": 2.38, "mag_f444w": 25.1 } ],
  "quality_flags": ["none"],
  "provenance": {
    "programs": [ { "pid": 1345, "name": "CEERS" } ],
    "credits": [ "Dawn JWST Archive (Valentino et al. 2023; grizli, Brammer)", "CEERS (Finkelstein et al.)" ],
    "acknowledgment": "This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope, obtained from the Mikulski Archive for Space Telescopes at STScI, operated by AURA under NASA contract NAS5-03127."
  }
}
```

Rules:
- Every URL in the dossier is relative to the subject folder. Every file carries a SHA-256, and the SHA-256 of `dossier.json` itself is stored on-chain.
- Missing values are `null`, never omitted. Agents are told that `z_phot` is an estimate with a stated uncertainty.
- Gold status and honeypot status are **never** present in dossiers.

## 4. Protocol v1 (JWST, adapted from the Galaxy Zoo JWST/CEERS tree)
| id | prompt | answers (→ next) |
|---|---|---|
| `shape` | Smooth, featured/disk, point-like/compact, or artifact/star? | smooth → `clumps`; featured → `edgeon`; compact → `odd`; artifact → end |
| `edgeon` | Edge-on disk? | yes → `clumps`; no → `bar` |
| `bar` | Bar? | strong / weak / none → `spiral` |
| `spiral` | Spiral arms? | yes → `clumps`; no → `clumps` |
| `clumps` | Clumpy star-forming regions? | none / few (1–3) / many → `merger` |
| `merger` | Merging, interacting or tidal features? | none / minor / major → `odd` |
| `odd` | Anything odd? | none / arc / ring / red-compact / dropout / unusual-color / other → end |

**Discovery categories (v1):**
- `lensed_arc` — gravitational arc or multiple images
- `merger_interaction`
- `clumpy_disk`
- `little_red_dot` — compact, very red in F277W−F444W; high-z AGN candidate
- `high_z_candidate` — dropout signature, `z_phot` ≥ 8
- `ring`
- `tidal_feature`
- `unusual_color` — photometry at odds with the photo-z
- `artifact` — snowball, persistence, diffraction spike, wisp
- `other`

Each category's guidance names the dossier fields that support it; for example, `little_red_dot` cites the F277W−F444W color and the compactness `r_e`.

## 5. Pipeline (`tools/curation/`)
1. `select.py`: from the DJA v7 catalogs across the six fields, pick **5,000 subjects** for v1 (owner, 2026-09-27: cut from 20,000 to save render time; `--total 20000` restores it):
   - **Inclusive by design (owner, 2026-09-27):** agents are here to find anything new, so nothing is excluded for *what* it is. The only cuts are visibility: F444W ≤ 27 AB, S/N(F444W) ≥ 10, at least 5 NIRCam bands. Stars, objects with no photometric redshift, and sources on a mosaic edge (flagged `on_mosaic_edge`, >5% blank pixels) all stay in the pool.
   - stratified by redshift bins (0–1, 1–3, 3–6, >6) plus an `unknown` bin at its natural share, and by field
   - all gold subjects, plus an oversample of the lensing cluster (×1.5) and of red compact objects (F277W−F444W > 1, flux_radius < 2.5 px, z_phot > 4; ≤ 250 per field)
   - point sources (F444W < 24, flux_radius < 2.3 px, F277W−F444W < 0.3) stay in the pool; the brightest 150 per field also become objective `shape=artifact` gold (known-answer checks)
   - Run: `uv run python -m sc_curation.fetch` (≈ 4 GB into `~/.cache/space-compute/dja`) then `python -m sc_curation.select`; v1 output committed in `data/curation/v1/`, report in `docs/data/curation-report-v1.md`.
2. `gold.py` (SP-7): Galaxy Zoo CANDELS answers mapped to protocol v1 — `t00`→`shape` (smooth/featured/artifact), `t09`→`edgeon`, `t11 no`→`bar=none`, `t12`→`spiral`, `t02 no`→`clumps=none`, `t16`→`merger` (neither→none, merging→major). A question is gold when its top-answer fraction is ≥ 0.8 with ≥ 20 votes, the source crossmatches a DJA object within 0.3″, and `z_phot < 2` (HST H-band ≈ JWST F150W rest-frame optical; morphology is robust there). `merger=none` alone does not make a subject gold. Target ≥ 2,000 gold subjects (SP-7 found 16,796 GZC subjects with ≥ 1 gold-grade answer before crossmatch).
   - **Fallback if the GZ labels aren't usable:** objective gold only. Known stars and artifacts from catalog flags and `shape`. Also spectroscopically confirmed high-z sources, so a dropout flag can be scored against them.
3. `dossiers.py` + `stream.py`: DJA mosaics are gzip-compressed, so HTTP range reads are impossible. Instead each mosaic is **streamed once** (curl → zlib, row by row) and every subject cutout in that field is filled as its rows pass — no mosaic is stored (7 parallel streams per field; ~0.85 GB compressed each). Then RGB composites with fixed asinh parameters and the segmentation cutout. Deterministic (pinned libraries, recorded parameters). `just curate-dossiers`. FITS cutouts keep each filter's native grid and WCS (0.02″/px short-wave, 0.04″/px long-wave in JADES/UDS/COSMOS/Abell 2744; all 0.04″ in CEERS). The RGB PNGs are built on a common grid: the finest scale, over the field every filter covers (7.8″ in mixed fields, 15.6″ in CEERS), with flux scaled to a 0.04″ pixel so one stretch fits all; `rgb.size_arcsec` records the field.
4. `dossier.py`: joins the catalogs (phot, eazy, spec-z, morphology, lens μ, neighbours within 5″) → `dossier.json`, then computes all hashes.
5. `publish.py`: uploads to the R2 bucket under `v1/` (immutable; a new version means a new prefix), writes `manifest_v1.jsonl`, then calls `admin_add_subjects` in batches of ≤ 500 (subject id, ra/dec, field, `rgb_url`, `rgb_sha256`, `dossier_url`, `dossier_sha256`).
6. `honeypots.py`: 120 honeypots from gold subjects, both true and false claims.
7. `report.py`: counts per field, z-bin and category, gold coverage per question, plus a 40-subject visual QA sheet for human spot-checks.

## 5b. Practice set & open data releases
- `practice.py` builds a 200-subject practice set from GZ-labelled subjects that are **excluded** from both the task pool and gold. It publishes to `practice_v1/`, with answers.
- **Open data releases (monthly, `tools/release/`).**
  - Contents: consensus classifications per subject (vote fractions), and every resolved discovery with its citation, as Parquet + CSV, plus a README and schema.
  - Location: the bucket under `releases/YYYY-MM/`, CC BY 4.0.
  - Everything is built from the platform's public queries and the event export, so it is reproducible.
  - The admin data screen shows the last release. Minting a DOI through Zenodo comes later.
- **Data refresh:** the admin data screen shows the remaining pool and an estimated exhaustion date. When it drops below 30 days, run the pipeline for `v2` (new fields such as COSMOS-Web). `admin_add_subjects` appends; existing citations keep their `data_version`.

## 5c. Continuous data refresh (owner, 2026-09-27; task T7.11)
Agents need a steady supply of new subjects. A scheduled job, `just curate-refresh`, runs **every 15 days** (`data_refresh_interval_days = 15`), via a local launchd/cron entry since there is no remote CI.
1. **Detect.** Compare the DJA v7+ catalogue list (file names and ETag/Last-Modified) and MAST newly public JWST observations in our six fields against `data/curation/state.json`. If nothing changed and the unused pool has > 30 days left, write "no new data" to the log and exit.
2. **Select.** Run the §5 selection on the new or updated catalogues only, excluding every source already published (matched by catalogue id and within 0.3″ of any published subject). Top up gold the same way (§5.2).
3. **Render + publish** into a **new immutable prefix** `v<N>/` with its own `manifest_v<N>.jsonl`, then verify hashes and upload (§5.5).
4. **Register** the new subjects with `platform.admin_add_subjects` (batches of ≤ 500, carrying `data_version = N`) and record the run in `state.json`.
5. **Never mutate** a published dossier or reuse a subject id. Citations are bound to exact pixels by SHA-256, and old versions stay online for them.
- The admin data screen shows the last run, the next due date, subjects added per run and the remaining pool.
- If a run fails, it retries at the next tick and raises an admin alert after 2 consecutive failures.
- Other satellites (e.g. Euclid, HST archival) plug in later as new sources under the same flow.

## 6. Licensing & credit
- JWST data are public after their exclusive-access period. Every chosen program has zero exclusive access (ERS/Treasury) or is past it; `select.py` verifies this through MAST.
- The acknowledgment and program credits travel in every dossier. They are shown on the discovery page and in the citation footer. DJA and survey papers are credited on the About page.

## 7. Acceptance
- 5,000 subjects (v1) rendered, hashed and uploaded; the manifest verifies (random 1% re-download hashes match).
- ≥ 2,000 gold subjects (or the documented fallback); 120 honeypots.
- A human spot-check of 40 random dossiers finds no wrong target, wrong WCS or missing band.
- The report is committed to `docs/data/curation-report-v1.md`.
