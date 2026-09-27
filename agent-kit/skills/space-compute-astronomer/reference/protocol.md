# JWST primer and protocol v1 guide

## 1. Reading the data

### NIRCam filters
Each subject has six NIRCam cutouts (`dossier.images.fits`), 10″ across at 0.0257″/px, north up.

| filter | pivot (µm) | PSF FWHM | what it traces |
|---|---|---|---|
| F115W | 1.15 | 0.04″ | rest-frame UV/blue light at z ≳ 1; vanishes for z ≳ 8.5 (Lyman break) |
| F150W | 1.50 | 0.05″ | young stars, star-forming clumps; blue channel of `rgb.png` |
| F200W | 1.99 | 0.07″ | sharpest mid-band; best for fine structure (`rgb_sw.png` is short-wave only) |
| F277W | 2.79 | 0.09″ | green channel; older stars at z ~ 1-3 |
| F356W | 3.56 | 0.12″ | stellar mass; Balmer break moves here at z ~ 7 |
| F444W | 4.42 | 0.15″ | red channel; dust, old stars, Hα/[OIII] at high z |

`rgb.png` = F150W (blue) / F277W (green) / F444W (red), asinh stretch. The long-wave filters have a 2-3x wider PSF, so compact objects look softer and redder in their cores. `rgb_sw.png` shows the short-wave filters at full sharpness: use it for clumps, bars, arcs, and thin tails.

### Redshift and colour
- Redshift moves every feature to longer wavelengths by (1+z). A normal galaxy at z ≈ 0.5 looks white/blue; at z ≈ 2-3 its old stars move into F277W-F444W and it looks orange; a dusty galaxy looks red at any z.
- `z_phot` is the photometric redshift; `z_phot_p16`/`z_phot_p84` bound the 68% interval. A wide interval (p84 − p16 > 1) means the redshift is poorly constrained: don't build a discovery on it. `z_spec` (when present) beats `z_phot`.
- Colours are magnitude differences: `F277W−F444W = mag_ab(F277W) − mag_ab(F444W)`. Positive = redder. > 1 mag is very red.
- A **dropout** is a source that is present in the red bands and absent (flux ≈ 0 or below `err`) in the bluest bands: the Lyman break has moved past them. Missing F115W and F150W with a clean detection redward is the z ≳ 10 signature; missing only F115W is z ≈ 8-9.
- `kpc_per_arcsec` converts sizes: `r_e_arcsec` × it = physical half-light radius. `r_e < 0.1″` at z > 4 is under ~0.7 kpc: compact.

### Common JWST artifacts (answer `shape = artifact`)
- **Diffraction spikes**: six-pointed star pattern (plus two faint horizontal ones) from a bright star; straight, symmetric, crossing the cutout.
- **Snowballs**: round, sharply edged blobs from cosmic-ray showers, present in one filter only (compare FITS bands).
- **Persistence**: a ghost of a previous bright source; appears in one exposure/filter, no colour gradient, no counterpart in other bands.
- **Wisps**: faint, large diffuse arcs of scattered light in the short-wave detectors, not centred on the target.
- **Mosaic edge** (`quality_flags` contains `mosaic_edge`, `target.on_mosaic_edge`): noise stripes, cut-off light, zero-filled regions. Classify what is visible; never flag edge noise as a discovery.
- Stars: point-like, diffraction spikes, `r_e` near the PSF size, featureless colour. Answer `artifact` (the option is "Artifact or star").

## 2. Protocol v1 decision guide
Walk from `shape`. Each answer names the next question; `null` ends the walk. Submit one answer per visited question, in order. Answer from the image; use the dossier to confirm, not to replace, what you see.

**`shape` - Smooth, featured/disk, point-like/compact, or artifact/star?**
- `smooth` → `clumps`: light falls off smoothly from the centre, elliptical/spheroidal, no disk, arms or knots.
- `featured` → `edgeon`: any disk, arm, bar, ring, clump or irregular structure resolved across several PSF widths.
- `compact` → `odd`: barely resolved or unresolved (`r_e_arcsec` ≲ 0.1″), but not a star (no spikes).
- `artifact` → end: star, spike, snowball, persistence, wisp, or the cutout is dominated by edge noise.

**`edgeon` - Edge-on disk?** `yes` → `clumps` if a thin, elongated disk (`axis_ratio` ≲ 0.3), often with a central bulge or dust lane. `no` → `bar`.

**`bar` - Bar?** `strong` / `weak` / `none` → `spiral`. A bar is a straight, elongated central feature distinct from the disk orientation. Faint or ambiguous = `weak`; most high-z disks are `none`.

**`spiral` - Spiral arms?** `yes` / `no` → `clumps`. Need at least one curved arm winding out from the centre, visible in `rgb_sw.png`.

**`clumps` - Clumpy star-forming regions?** `none` / `few` (1-3) / `many` → `merger`. Clumps are compact bright knots (usually blue/white) inside the galaxy's light, not separate neighbours. Very common at z > 1.

**`merger` - Merging, interacting or tidal features?** `none` / `minor` / `major` → `odd`. `major`: two comparable galaxies overlapping or linked by bridges. `minor`: a small companion touching the main body, or faint tails. Check `neighbours` (sep < 1″ with a visible link supports it); a close neighbour with no visible disturbance is `none` (it may be a projection).

**`odd` - Anything odd?** → end.
- `none`: default.
- `arc`: thin curved streak tangential to a bright foreground galaxy or cluster (`lensing.magnification` > 1 supports it).
- `ring`: closed or near-closed ring of light around a centre.
- `red-compact`: compact and very red (F277W−F444W ≳ 1 mag) with blue-band flux low.
- `dropout`: absent in the bluest bands, present redward (see §1).
- `unusual-color`: colour clearly at odds with the photo-z (e.g. very red at `z_phot` < 1 with no dust signature).
- `other`: anything else you would want a second look at; say why in a flag rationale or leave unflagged.

## 3. Discovery categories
Flag only clear cases. Cite visible evidence and the dossier fields named here.

| id | what it looks like | cite |
|---|---|---|
| `lensed_arc` | curved arc(s) or multiple images around a foreground lens | arc geometry, `lensing.magnification`, lens `z_phot` |
| `merger_interaction` | two or more galaxies overlapping, bridges, disturbed shapes | `neighbours` sep, both components visible |
| `clumpy_disk` | disk dominated by giant star-forming clumps | clump count, `z_phot`, F150W brightness of clumps |
| `little_red_dot` | point-like, very red in F277W−F444W; high-z AGN candidate | F277W−F444W colour, `r_e_arcsec`, `z_phot` ± interval |
| `high_z_candidate` | dropout signature with `z_phot` ≥ 8 | blue-band flux vs `err`, `z_phot_p16` ≥ ~7 |
| `ring` | ring galaxy or ring-like structure | ring radius, centre, colour |
| `tidal_feature` | tidal tails, shells or streams | length/direction, `neighbours` |
| `unusual_color` | photometry at odds with the photo-z | the colours and `z_phot` that conflict |
| `artifact` | snowball, persistence, diffraction spike or wisp worth recording | band(s) it appears in |
| `other` | anything else worth a second look | what and why |

## 4. Common mistakes
- Calling a star `compact`: stars have diffraction spikes and white colour; use `artifact`.
- Calling a neighbour a clump, or a projection a merger: look for a physical link.
- Flagging `little_red_dot` from the RGB alone: check the actual F277W−F444W from `photometry.bands` and `r_e_arcsec`.
- Flagging `high_z_candidate` with a wide `z_phot` interval or a blue-band detection above `err`.
- Flagging noise on a `mosaic_edge` cutout.
- Skipping questions: the answers must follow the tree exactly or the submission is rejected with `InvalidInput`.
