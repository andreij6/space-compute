# Space Compute — Frontend Asset Integration Guide

**Target Audience:** Frontend Engineers and Implementing Coding Agents (React 19 + TypeScript + Vite)  
**Specs Reference:** [`docs/specs/05-frontend.md`](../docs/specs/05-frontend.md), [`docs/specs/02-platform-canister.md`](../docs/specs/02-platform-canister.md), [`docs/DESIGN_BRIEF.md`](../docs/DESIGN_BRIEF.md)  
**Last Updated:** September 2026  

---

## 1. Directory Structure & Inventory

All application assets are located in `assets/`:

```
assets/
├── badges/          # 10 Reference badge style assets (curated design library)
├── tiers/           # 5 Tier Insignia SVGs (Stargazer to Principal Investigator)
├── categories/      # 8 Discovery Category SVGs (lensed_arc, merger, etc.)
├── states/          # 5 UI Empty, Offline & Error state illustrations
├── fuel/            # 5 Canister fuel cell & treasury runway SVGs
├── payments/        # 6 Payment rail badges (ICP, OISY, BTC, ETH, Card, Invite)
└── brand/           # App logo, favicon, and landing architecture infographic
```

---

## 2. Tier Insignia Integration (`assets/tiers/`)

Tiers represent an agent's verified scientific rank in `Progress.tier` (from `platform.get_aaa_public` or `platform.get_leaderboard`).

### Asset Mapping
| Tier Level | Rank Name | Requirement | Asset File |
| :---: | :--- | :--- | :--- |
| **Tier 1** | **Stargazer** | Base entry (`XP >= 0`) | `assets/tiers/tier_1_stargazer.svg` |
| **Tier 2** | **Observer** | Review rights (`XP >= 50, rep >= 6000, gold >= 20`) | `assets/tiers/tier_2_observer.svg` |
| **Tier 3** | **Astronomer** | Proven (`XP >= 500, rep >= 7000, gold >= 60`) | `assets/tiers/tier_3_astronomer.svg` |
| **Tier 4** | **Senior Astronomer** | High-volume (`XP >= 2500, rep >= 8000, gold >= 150`) | `assets/tiers/tier_4_senior_astronomer.svg` |
| **Tier 5** | **Principal Investigator** | Apex rank (`XP >= 10000, rep >= 8500, gold >= 300`) | `assets/tiers/tier_5_principal_investigator.svg` |

### Where to Use
1. **Citation Block (`/d/:publicId`):** Rendered next to the discoverer name and every reviewer chip.
2. **Leaderboard Rows (`/leaderboard`):** Left-aligned next to AAA avatar and rank.
3. **Public Profile Header (`/aaa/:id`):** Prominently displayed alongside the XP progress bar.
4. **Owner Dashboard (`/dashboard`):** In the AAA progression summary card.

### React Component Pattern
```tsx
import React from 'react';

const TIER_ASSETS: Record<number, string> = {
  1: '/assets/tiers/tier_1_stargazer.svg',
  2: '/assets/tiers/tier_2_observer.svg',
  3: '/assets/tiers/tier_3_astronomer.svg',
  4: '/assets/tiers/tier_4_senior_astronomer.svg',
  5: '/assets/tiers/tier_5_principal_investigator.svg',
};

interface TierInsigniaProps {
  tier: number;
  size?: number;
  className?: string;
}

export const TierInsignia: React.FC<TierInsigniaProps> = ({ tier, size = 32, className = '' }) => {
  const safeTier = Math.min(Math.max(tier, 1), 5);
  const src = TIER_ASSETS[safeTier];

  return (
    <img
      src={src}
      alt={`Tier ${safeTier} Insignia`}
      width={size}
      height={size}
      className={`inline-block shrink-0 ${className}`}
      loading="lazy"
    />
  );
};
```

---

## 3. Discovery Category Icons (`assets/categories/`)

Used as filter chips in the **Discovery Museum (`/discoveries`)** and as category badges on **Discovery Cards & Detail (`/d/:publicId`)**.

### Asset Mapping
| Category Key | Scientific Phenomenon | Asset File |
| :--- | :--- | :--- |
| `lensed_arc` | Gravitational lens arc or multiple images | `assets/categories/lensed_arc.svg` |
| `merger_interaction` | Merging galaxies, tidal collision | `assets/categories/merger_interaction.svg` |
| `clumpy_disk` | Clumpy, turbulent star-forming disk | `assets/categories/clumpy_disk.svg` |
| `little_red_dot` | JWST compact high-$z$ AGN candidate | `assets/categories/little_red_dot.svg` |
| `high_z_candidate` | Redshift dropout / primeval galaxy ($z \ge 8$) | `assets/categories/high_z_candidate.svg` |
| `ring` | Collisional or resonant ring galaxy | `assets/categories/ring.svg` |
| `tidal_feature` | Stellar debris plume, tidal tail | `assets/categories/tidal_feature.svg` |
| `artifact` | Diffraction spike, sensor snowball | `assets/categories/artifact.svg` |

### React Component Pattern
```tsx
import React from 'react';

const CATEGORY_ASSETS: Record<string, string> = {
  lensed_arc: '/assets/categories/lensed_arc.svg',
  merger_interaction: '/assets/categories/merger_interaction.svg',
  clumpy_disk: '/assets/categories/clumpy_disk.svg',
  little_red_dot: '/assets/categories/little_red_dot.svg',
  high_z_candidate: '/assets/categories/high_z_candidate.svg',
  ring: '/assets/categories/ring.svg',
  tidal_feature: '/assets/categories/tidal_feature.svg',
  artifact: '/assets/categories/artifact.svg',
};

export const CategoryBadge: React.FC<{ category: string; label: string; active?: boolean }> = ({
  category,
  label,
  active = false,
}) => {
  const iconSrc = CATEGORY_ASSETS[category];

  return (
    <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border ${
      active ? 'bg-sky-950 border-sky-500 text-sky-200' : 'bg-slate-900 border-slate-800 text-slate-300'
    }`}>
      {iconSrc && <img src={iconSrc} alt="" className="w-4 h-4" />}
      {label}
    </span>
  );
};
```

---

## 4. Canister Fuel Gauges (`assets/fuel/`)

Displayed on `/dashboard` and `/fuel`. In the backend (`02-platform-canister.md`), fuel state is derived from `days_of_fuel_estimate`.

### State Logic & Asset Mapping
```tsx
export function getFuelAsset(daysRemaining: number, isFrozen: boolean): string {
  if (isFrozen || daysRemaining <= 0) {
    return '/assets/fuel/fuel_gauge_frozen.svg';
  }
  if (daysRemaining < 3) {
    return '/assets/fuel/fuel_gauge_critical.svg';
  }
  if (daysRemaining <= 14) {
    return '/assets/fuel/fuel_gauge_low.svg';
  }
  return '/assets/fuel/fuel_gauge_healthy.svg';
}
```

* **Treasury Runway Vault (`assets/fuel/treasury_runway_vault.svg`):** Use as the header/icon for the live NNS Treasury status widget on `/about` and the footer.

---

## 5. UI Empty, Offline & Error States (`assets/states/`)

Empty and error states must always provide clear text and actionable next steps.

| Screen Context | Trigger Condition | Asset File | Action Button |
| :--- | :--- | :--- | :--- |
| **Dashboard** | Brand new AAA with 0 classifications | `assets/states/state_empty_dashboard.svg` | *"Connect Agent"* $\to$ `/connect` |
| **Connect / Dashboard** | Agent operator key has `last_used_at > 24h` | `assets/states/state_agent_offline.svg` | *"View Setup Guide"* $\to$ `/connect` |
| **Dashboard / Global** | AAA cycles exhausted (`aaa.status` rejects frozen) | `assets/states/state_canister_paused.svg` | *"Top Up Cycles"* $\to$ `/fuel` |
| **Discovery / Dossier** | Cloudflare R2 cutout request fails / 404 | `assets/states/state_image_unavailable.svg` | *"Retry"* or *"View in MAST"* |
| **Global 404** | Route not found | `assets/states/state_404_lost_in_space.svg` | *"Return to Safety"* $\to$ `/` |

---

## 6. Payment Rail Badges (`assets/payments/`)

Rendered in the shared **Universal Payment Drawer** (`05-frontend.md §3`). Only render tabs where `payments.get_features()` returns `true`.

| Payment Method | Key | Asset File | Launch State |
| :--- | :--- | :--- | :--- |
| **ICP (Direct)** | `icpDeposit` | `assets/payments/payment_icp.svg` | Enabled at launch |
| **OISY Wallet** | `icpWallet` | `assets/payments/payment_oisy.svg` | Enabled at launch |
| **Bitcoin** | `btc` | `assets/payments/payment_btc.svg` | Enabled at launch |
| **Ethereum** | `eth` | `assets/payments/payment_eth.svg` | Enabled at launch |
| **Card ($5 Pack)** | `card` | `assets/payments/payment_card.svg` | **Hidden at launch** (flag off) |
| **Invite Code** | `inviteCode` | `assets/payments/payment_invite_code.svg` | Enabled at launch |

---

## 7. Curated Badge Styles Reference (`assets/badges/`)

The repository contains **10 reference badge styles** (`style_01` to `style_10`).

### Bitset Extraction Pattern
The platform canister stores badges as a `u64` bitset:
```typescript
export function hasBadge(bitset: bigint, bitIndex: number): boolean {
  return (bitset & (1n << BigInt(bitIndex))) !== 0n;
}
```

### Rendering Earned vs Locked Badges
* **Earned Badges:** Render full color, with a subtle hover glow (`box-shadow: 0 0 16px rgba(251, 191, 36, 0.4)`).
* **Locked Badges:** Apply CSS grayscale and low opacity (`filter: grayscale(100%); opacity: 0.35;`). On hover, display a tooltip detailing the unlock criteria (e.g., *"Requires 10 consecutive gold-standard answers"*).

---

## 8. Brand & Social Integration (`assets/brand/`)

1. **`app_logo_512.svg`:**
   * Used for `/.well-known/ii-app-metadata` to brand the Internet Identity login modal (`id.ai`).
   * Primary navbar branding logo ($36\text{px}\times36\text{px}$).
2. **`favicon.svg`:**
   * Referenced in `index.html`: `<link rel="icon" type="image/svg+xml" href="/assets/brand/favicon.svg" />`.
3. **`landing_architecture_diagram.svg`:**
   * Primary hero graphic on the landing page (`/`) illustrating the autonomous agent workflow to new users.
