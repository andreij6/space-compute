---
type: runbook
title: "Ops Runbook — Space Compute"
tags: [operations]
timestamp: 2026-09-27T01:20:00-04:00
---

# Ops Runbook — Space Compute

Operational reference for deployed Space Compute canisters (`platform`, `payments`, `treasury`, `frontend`, and spawned `aaa` canisters). See `DEPLOY.md` for deployment procedures.

## 1. Cycles Management & Dual Treasury

Space Compute operates a dual-treasury architecture to ensure 100% autonomous cycles runway without relying on manual top-ups:

### A. Ops Treasury (`treasury` canister)
- **Funding:** Owner-funded ICP float (recommended float: 10–20 ICP, providing ~3–6 months runway).
- **Automation:** The `treasury` canister runs a 6-hour periodic timer:
  1. Reads cycle balances of `platform`, `payments`, `frontend`, and itself via `ic_cdk::api::canister_balance()`.
  2. If any canister falls below its 30-day burn threshold, it transfers ICP to CMC (`rkp4c-7iaaa-aaaaa-aaaca-cai`) and invokes `notify_top_up`.
  3. Exposes `health() -> TreasuryHealth` for real-time runway monitoring.
- **Reserve Floor:** Never tops up below the 5 ICP hard reserve floor.

### B. Fuel Treasury (`payments` canister `TREASURY` subaccount)
- Backs user fuel packs (card, BTC, ETH) paid with fiat/crypto.
- Refilled from Stripe revenue and ops treasury allocations.
- Auto-pauses non-ICP intake if the reserve floor is reached.

### Canister Freezing Thresholds & Cycle Targets

| Canister | Freezing Threshold | Alert Target | Min Controllers |
|---|---|---|---|
| `platform` | 90 days (`7776000`s) | < 10 T cycles | 3 (team, backup, treasury) |
| `payments` | 90 days (`7776000`s) | < 10 T cycles | 3 (team, backup, treasury) |
| `frontend` | 30 days (`2592000`s) | < 5 T cycles | 3 (team, backup, treasury) |
| `treasury` | 180 days (`15552000`s) | < 15 T cycles | 2 (team hardware key, backup) |
| `aaa` (template) | 60 days (`5184000`s) | < 2 T cycles | 2 (owner principal, platform) |

---

## 2. Controllers & Governance Hygiene

Every value-holding and stateful canister must have at least 2 controllers before mainnet launch.

`scripts/treasury-watch.sh <env>` (`just watch-treasury <env>`) automates the treasury half of
this: it adds `platform`/`payments`/`frontend`/`treasury` to the treasury canister's watch
list (`admin_watch`) and adds `treasury` as a co-controller of `platform`/`payments`/`frontend`
(needed for `canister_status` reads). It is idempotent — safe to re-run. The human backup
controller is still added by hand:

```bash
# Add backup controller
icp canister settings update <canister> --add-controller <BACKUP_PRINCIPAL> -e production --identity <identity>

# Verify controllers
icp canister status <canister> -e production --identity <identity>
```

Recommended controller configuration:
- `treasury`: Team hardware wallet + offline cold backup principal.
- `platform` & `payments`: Team hardware wallet + backup principal + `treasury` canister principal (enables autonomous cycle checks and top-ups).
- `aaa`: User owner principal + `platform` canister principal (for managed wasm upgrades).

### Cycles monitoring script

`scripts/cycles-report.sh <env>` (`just cycles <env>`) prints per-canister cycles, burn/day,
and runway (from `icp canister status`) plus `treasury.health()`, and exits non-zero if any
canister is under `CYCLES_ALERT_DAYS` (default 30) or the treasury reserve is breached — wire
it into cron or launchd (`scripts/com.spacecompute.cycles-report.plist` is a launchd
template) for the "every 6h, alert below 30 days of burn" monitor in
`docs/specs/09-testing-ops.md` §3.

---

## 3. Emergency Incident Procedures

### A. Emergency Canister Pause
To pause intake and task dispatch in case of suspected anomaly or exploit:

```bash
# Pause platform tasks and reviews
icp canister call platform admin_set_paused '(true)' -e production --identity <ADMIN_KEY>

# Pause non-ICP payments
icp canister call payments admin_pause_non_icp '(true)' -e production --identity <ADMIN_KEY>

# Full payments pause
icp canister call payments admin_set_paused '(true)' -e production --identity <ADMIN_KEY>
```

### B. Manual Canister Top-Up (Direct Emergency Intervention)
If treasury is exhausted or CMC fails:

```bash
# Check cycles balance
icp canister status <canister_name> -e production

# Top up cycles directly via wallet
icp wallet send <canister_principal> <cycles_amount>
```

### C. Post-Wipe Local Identity Permutation (Dev Environment)
On local network reset, canister IDs permute. Never hardcode IDs. Run:

```bash
bash scripts/deploy-local.sh
```
This script re-captures live canister IDs into `.env.local/canister_ids.env` and rewires inter-canister calls idempotently.
