# 05a — Design reconciliation (T9.1)

This file records where the Phase 9 look comes from and how it maps onto the behaviour spec (`05-frontend.md`). The spec defines behaviour and data. The design defines look and layout. Where they disagree, the spec wins on behaviour, roles, labels and text, because the Playwright selectors depend on those.

## 1. Sources

| Source | Status | Used for |
|---|---|---|
| Claude Design project `c2cf5a8d-5b7d-4280-9bab-0b6a76cb4619` (`Space Compute Mockups.dc.html`, `Discovery Museum.dc.html`, `support.js`) | **Not available locally** (no `/design-login` session, no export in `docs/design/`). Not used. | Pending: see §6 |
| `docs/DESIGN_BRIEF.md` | Used | Screen list, states, tone ("earned, not childish", "images are the star", explain cycles as fuel) |
| `docs/design-screens.json` (25 screens, 4 shared components) | Used | Key elements per screen, and the COMP-01…04 component list |
| `assets/` + `assets/ASSET_INTEGRATION_GUIDE.md` | Used | Tier insignia, category icons, state illustrations, fuel gauges. Payment badges are left for T9.3 (payment component). Brand files are already in `public/` |
| The external agent's styled mock UI (`git show 32a300a:src/frontend/src/index.css` and its pages) | Used as the visual starting point | Dark "space" palette, amber/cyan accents, radii, card and button look, navbar, drawer, bottom bar and footer layout |

## 2. Token decisions (`src/frontend/src/styles/tokens.css`)

- **One theme, dark.** The brief and the old mock are both dark "deep space". `color-scheme: dark` is set. There is no light theme; adding one later only means redefining the `--color-*` tokens.
- **Colour.** Values are taken from the old mock and renamed semantically:

  | Old mock name | Token |
  |---|---|
  | `--bg-space` | `--color-bg` |
  | `--bg-surface-elevated` | `--color-surface-raised` |
  | `--amber-star` | `--color-accent` |
  | `--cyan-nebula` | `--color-success` |
  | `--red-nova` | `--color-danger` |
  | `--blue-cosmic` | `--color-info` |

  Other colour changes:
  - `--text-dim` was lifted from `#7b8aa6` (4.85:1 on the raised surface) to `#8593ad` (5.44:1) for margin.
  - Badge fills (`*-soft`) are opaque, so their contrast can be computed. The old mock used translucent glows.
  - The old names stay in `index.css` as aliases, because the unstyled Phase 6 pages still use them inline. T9.2–T9.4 remove the aliases as pages migrate.
- **AA contrast.** `src/styles/tokens.test.ts` parses `tokens.css` and asserts ≥ 4.5:1 for every text token on every surface, on-accent on accent, and each tone on its soft fill. The lowest pair is text-dim on the raised surface at 5.44:1.
- **Type.**
  - Display font: Space Grotesk. Body font: Inter. Mono font: JetBrains Mono. These are the old mock's fonts, each with a system fallback stack.
  - **No web fonts are loaded**, so the spec's rule of no third parties still holds. Self-hosting them is a T9.5 decision against the bundle budget.
  - Scale: xs 12, sm 14, md 16, lg 18, xl 22, 2xl 28, 3xl 36 px.
- **Spacing:** a 4-based scale, `--space-1…8` = 4, 8, 12, 16, 24, 32, 48 and 64 px.
- **Radius:** 6, 10, 16 px and full (from the old mock).
- **Shadow:** card, glow and dialog.
- **Motion:** 150/200 ms with a standard easing curve. Both durations drop to 0 under `prefers-reduced-motion`.
- **Layout:** max width 1280 px, as in the old mock.

## 3. Shared components (CSS modules)

| Component | File | Derived from |
|---|---|---|
| Button / ButtonLink (primary, secondary, danger, ghost; sm/md; `busy` → `aria-busy` + disabled, for the 05 §3 pending state) | `components/ui/Button.tsx` | Old mock `.btn-primary/.btn-secondary`. Danger and ghost are new, for admin actions |
| Card (a titled card is a `section` labelled by its `h2`) | `components/ui/Card.tsx` | Old mock `.card` |
| Badge (neutral/accent/success/danger/info), TierInsignia (`Tier n: Name` alt text) | `components/ui/Badge.tsx` | Old mock `.badge-*`, `assets/tiers`, `progression.ts` names |
| CategoryIcon (platform category ids mapped to `assets/categories`) | `components/ui/CategoryIcon.tsx` | Asset guide §3. The platform ids (`lens`, `merger`, `red_dot`, `clumpy`, `tidal`) differ from the asset file names; the component maps them |
| EmptyState (not_found, canister_paused, agent_offline, image_unavailable, empty_dashboard) | `components/EmptyState.tsx` | Asset guide §5 illustrations. The old mock's text, headings and actions are kept unchanged |
| FuelGauge (Healthy > 14 d, Low 3–14 d, Critical < 3 d, Paused when frozen; a labelled group with a `meter`) | `components/ui/FuelGauge.tsx` | Asset guide §4, COMP-03, 05 §3. It replaces the unused `FuelCellGauge` |
| PageShell (skip link, `main#main`) + Navbar ("Primary"/"Menu" navs, `aria-current`, `aria-expanded`) + Footer + MobileBottomBar ("Quick" nav) | `components/PageShell.tsx` etc. | Old mock layout. Every link and button text is unchanged |
| DataTable (caption, `th scope=col`, numeric columns, empty row) | `components/ui/DataTable.tsx` | New, for the leaderboard and admin tables |
| Tabs (ARIA tabs; arrow keys skip disabled tabs) | `components/ui/Tabs.tsx` | New, for payment methods (COMP-01) and the admin views |
| Dialog (native `<dialog>` + `showModal`, labelled by its `h2`) | `components/ui/Dialog.tsx` | New, for the fuel refuel modal (COMP-01 usage) and the admin drawer |
| ConfirmAction (restyled; still inline; label and button names unchanged) | `components/ConfirmAction.tsx` | 05 §2b typed confirmation |

**Wiring.** Only the layout is wired in (RootLayout → PageShell). Pages are restyled in T9.2–T9.4. `/design` is a dev-only gallery, reached through `import.meta.env.DEV`. It is absent from production routes and from `dist`.

**Assets.** The needed SVGs are copied into `src/frontend/src/assets/{tiers,categories,states,fuel}`. They ship as separate hashed files (`assetsInlineLimit: 0`), so none of them lands in the JS bundle. The initial JS is 193.9 KB gz, against the 350 KB gate.

## 4. Per-screen notes

In the table, "Brief" means `DESIGN_BRIEF.md` and `design-screens.json`, and "Mock" means the old styled UI at `32a300a`. The last column is the task that restyles the screen.

| Screen | Route | Brief | Mock | Restyle in |
|---|---|---|---|---|
| PUB-01 Landing | `/` | Hero, how it works, live stats, recent confirmed discoveries, Spawn CTA; hero graphic `landing_architecture_diagram.svg` | Dark hero, amber CTA, card grid | T9.2 |
| PUB-02 Discovery Museum | `/discoveries` | Grid with category and status filters. Category chips use CategoryIcon | Card grid, badge chips | T9.2 |
| PUB-03 Discovery detail | `/d/:publicId` | The image is the star: zoom, data panel, prestigious citation block (COMP-02) with TierInsignia and vote chips | `.citation-block` with amber rule and mono text | T9.2 |
| PUB-04 AAA profile | `/aaa/:id` | TierInsignia + XP bar, earned and locked badges (grayscale, 0.35 opacity per the asset guide) | Card layout | T9.2 |
| PUB-05 Leaderboard | `/leaderboard` | Simple ranking. DataTable with TierInsignia | Table in a card | T9.2 |
| OWN-01 Spawn | `/spawn` | Wizard; payment component COMP-01 with the Tabs component | Stepper cards | T9.3 |
| OWN-02 Dashboard | `/dashboard` | "What happened while I was away"; FuelGauge; EmptyState `empty_dashboard` and `agent_offline` | `grid-dashboard` 320 px sidebar | T9.3 |
| OWN-03 Connect | `/connect` | Steps with copy buttons, operator table (DataTable) | Mono command blocks | T9.3 |
| OWN-04 Records | `/records` | Filter Tabs and a read-only detail panel | Card list | T9.3 |
| OWN-05 Fuel & billing | `/fuel` | FuelGauge, COMP-01, auto top-up, history DataTable | Gauge card, amber CTA | T9.3 |
| INFO-01…05 About / Practice / Terms / Privacy / Credits | `/about` … | Text-heavy; runway widget uses `treasury_runway_vault.svg` | Plain cards | T9.4 |
| ADM-01…11 Admin console | `/admin/*` | Functional over pretty. Left nav (AdminNav), DataTable, ConfirmAction, Dialog drawer | Not in the mock beyond the navbar link | T9.4 |
| Global 404 / empty states | `*` | COMP-04 illustrations | Lucide icons (now replaced by the asset illustrations) | done (T9.1) |

## 5. Mismatches found (spec wins)

- The asset guide suggests importing assets from `/assets/...` URLs and using Tailwind classes. The spec says CSS modules. Assets are imported through Vite, and the styles are CSS modules.
- `design-screens.json` COMP-01 lists Card as a tab. At launch the spec hides the card tab entirely (05 §3); it is not greyed out. The gallery shows a disabled Card tab only to demonstrate the disabled-tab style.
- The asset guide maps OISY and ICP as separate badges. The spec's strategy map is `{icpWallet, icpDeposit, card, btc, eth}` plus the invite code. T9.3 decides the icon per strategy.
- `high_z_candidate.svg` has no platform category yet. CategoryIcon accepts `high_z` for when the category is added.

## 6. Pending: compare against the Claude Design mockups

- [ ] Owner runs `/design-login`, or exports `Space Compute Mockups.dc.html`, `Discovery Museum.dc.html` and `support.js` into `docs/design/`.
- [ ] Diff the mockup palette, type and spacing against `tokens.css`. Update the tokens. Re-run `tokens.test.ts`, which must stay ≥ 4.5:1.
- [ ] Diff each component in §3 against its mockup counterpart (buttons, cards, badges, tier insignia, citation block, payment component, fuel cell, empty states).
- [ ] Compare the priority screens: Dashboard, Fuel & billing, Discovery detail, AAA profile, Activity & records, Landing.
- [ ] Record every look/layout mismatch in §5 above before starting T9.2.
- [ ] Decide on web fonts (self-host or system stack) against the 350 KB gz budget.

## 7. Responsive decisions (T9.5)

Mobile-first: base styles target the narrowest screen, `min-width` queries add the wider layout. Breakpoints: **480** (brand text), **640** (image-viewer hint), **768** (tablet: bottom bar off, admin side nav, compact `sm` controls), **900** (desktop primary nav). The target is no horizontal page scroll at **360 px**.

| Area | Decision |
|---|---|
| Navigation | Below 768 px: the fixed bottom bar ("Quick" nav: Home, Museum, Spawn, Agent, Ranks) plus the menu drawer. The navbar's Spawn button is hidden there, because the bar already has Spawn. The brand text is hidden below 480 px, and the logo alt keeps the link name. Primary links show from 900 px. The bar and the footer pad for `env(safe-area-inset-bottom)` |
| Tap targets | ≥ 44 px on mobile: buttons (`md` always 44; `sm` 44×44 below 768, then 32), tabs, inputs, Museum chips, drawer, bottom-bar, admin-nav and footer-list links, the menu button |
| Tables | Horizontal scroll, not stacked cards. Each scroll wrapper is a focusable `role="region"` named by the table caption or section, so keyboard and screen-reader users can scroll it (axe `scrollable-region-focusable`). Header cells don't wrap. Stacked cards were rejected because they would change the `cell`/`columnheader` roles the e2e tests use |
| Grids | Every `minmax(N px, 1fr)` with N ≥ 200 becomes `minmax(min(N px, 100%), 1fr)`, so no grid forces overflow. Long mono strings (`code`, `dd`, citation text, operator principals) use `overflow-wrap: anywhere`. Flex rows with two ends (toolbars, list items, data rows, the operator row, the deposit row, the fuel gauge) wrap |
| Headings | `h1` and page titles scale with `clamp(2xl, 6vw, 3xl)` |
| Citation block | The header and the copy button wrap. The citation text breaks anywhere. The verification badge stays in a `role="status"` live region |
| Image viewer | When not zoomed, `touch-action: pan-x pan-y`, so a one-finger swipe scrolls the page and double-tap reaches `dblclick` (toggle 2×). When zoomed, `touch-action: none`, and a drag pans the image. Wheel zoom needs Ctrl/⌘ (a trackpad pinch sends ctrl+wheel), so it never traps page scroll. It uses a non-passive native listener, so `preventDefault` works. The zoom buttons are 44 px. The hint shows only at ≥ 640 px with hover. The image height is capped at `min(520px, 70vh)` |
| Payment panel | Single column. The deposit account input and the copy button wrap. The amount, fee and busy/disabled rules are unchanged |
| Admin | Below 768 px, the admin nav is a horizontally scrolling row of 44 px links. From 768 px (tablet), it is the 200 px side column and the content column scrolls tables in place |
| Reduced motion | The tokens already zero `--duration-*`. A global `prefers-reduced-motion` rule also cuts every transition/animation to 0.01 ms and turns off smooth scroll |

**Regression gates (these run in the final verify, not per task; owner 2026-09-28).**
- `tests/e2e/visual.spec.ts`: `toHaveScreenshot` for every route (signed-out, spawn, owner, admin) at 390×844 and 1280×850. Dynamic data is masked. It also asserts that the page has no horizontal scroll at 360 px and that there are no tap targets under 44 px at 390 px. Create the baseline with `scripts/visual-baseline.sh --update` (skill `visual-baseline`) and commit it.
- `scripts/lighthouse-a11y.sh`: accessibility ≥ 90 and performance ≥ 80 on Landing, the first real Discovery detail and Dashboard. Reports go to `docs/demos/T9.5/`.

**Web fonts (the §6 open item).** Still the system stack. No font files are shipped, so the bundle and LCP don't change.

## 8. Review of T9.2–T9.4 (Opus, T9.5)

The money UI is correct: the fee is shown, units are ICP, buttons are busy/disabled while an op is pending, and a single `confirming` flag prevents a double submit. Other correct items: the citation fails closed; images are sha256-verified before render; every admin destructive action keeps typed confirmation; the draft legal banners are present; the e2e roles, labels and texts are unchanged. Defects fixed in the review commit:
1. Dashboard (and Fuel) showed the cycles twice, in the gauge and in the `dd`. That broke `getByText(/T Cycles/)` in strict mode. The gauge no longer repeats the cycles.
2. The citation verification badge had lost `role="status"`, so the async Verified/Unverified result wasn't announced. It is restored, with a unit test.
3. The dossier's FITS links were rendered without a scheme check. They now go through `safeHref` (https, or the dossier's own origin), with a unit test.
4. The image viewer's wheel `preventDefault` ran in a passive React listener (a no-op plus a console warning). `touch-action: none` also blocked page scroll on touch. The viewer hint used dim text on a translucent overlay (contrast not guaranteed).
5. The heading order skipped a level (`h1` → `h3` on Discovery detail and in the citation block; `h4` in the footer).
6. The Museum filter chips showed their selected state by colour only. They now have `aria-pressed`.
7. Locked AAA badges used 45 % opacity on the whole card, text included, which failed AA.
8. Scrollable table wrappers weren't keyboard-focusable.
9. CSS module hygiene: admin pages imported `ConfirmAction.module.css` for their form fields. Those styles now live in `styles/adminShared.module.css`. The dead legacy classes and colour aliases in `index.css` were removed.
