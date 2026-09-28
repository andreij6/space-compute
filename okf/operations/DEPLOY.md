---
type: runbook
title: "Deployment Runbook — Space Compute"
tags: [operations, deployment]
timestamp: 2026-09-27T01:20:00-04:00
---

# Deployment Runbook — Space Compute

Deployment reference for Space Compute across `local`, `staging`, and `production` environments using the `icp` CLI.

## 1. Local Deployment (Deterministic One-Shot)

Local deployment is 100% deterministic and managed via script:

```bash
# Recommended one-shot command
bash scripts/deploy-local.sh

# Or via Justfile
just deploy-local
```

### What it does:
1. Starts managed local network if not already running.
2. Installs ICRC test ledgers (`ledger`, `ckbtc-ledger`, `cketh-ledger`) once (Init mode, never upgraded).
3. Deploys core canisters (`treasury`, `payments`, `platform`, `frontend`).
4. Compiles and uploads the `aaa` template wasm to `platform`.
5. Wires inter-canister connections (`payments` -> `platform`, `treasury` -> watched canisters).
6. Seeds dev faucet with 100 ICP test funds.
7. Outputs `.env.local/canister_ids.env`.

---

## 2. Pre-Deploy Verification Gates

Run the local verification suite before any deployment or task completion:

```bash
# Full local verification (cargo fmt, clippy, unit/integration tests, wasm size)
bash scripts/verify-local.sh

# Or via Justfile
just verify
```

- Disallow test skips from masking failures (`L-007`).
- Verify AAA wasm size budget $\le 1.5$ MiB gz (`L-012`).

---

## 3. Staging & Production Deployment Protocol (T8.1)

Staging and production deploys go through `scripts/deploy-env.sh <env>` (`just release <env>`),
never ad-hoc `icp deploy`. It is the single release workflow for every environment,
including `local`; see `docs/ops/deploy-runbook.md` for the full first-staging-deploy
walkthrough.

```bash
export SC_ALLOW_MAINNET=1        # only when the owner asked this session
export DEPLOY_IDENTITY=<named identity>   # never the machine default (prod-deployer)
just release production          # or: bash scripts/deploy-env.sh production
```

It enforces, in order: `just verify` green, clean git tree, on `main`, an explicitly named
non-default identity, and a typed confirmation of the environment name (all skipped for
`local`, which is a dev loop, not a release). It then snapshots every canister about to be
upgraded, deploys, wires `platform.payments_id` <-> `payments.platform_id`, optionally
registers/approves the AAA template wasm (`AAA_REGISTER=1`), smoke-tests every canister
(`version()`, a public query, an HTTP GET for `frontend`), and rolls back automatically on
any smoke failure (§4). Add controllers and fund the treasury float right after the first
successful deploy — see `docs/ops/deploy-runbook.md` §3.

---

## 4. Rollback & Disaster Recovery

`scripts/deploy-env.sh` snapshots every canister it is about to upgrade
(`icp canister snapshot create`) before installing, and restores that snapshot
(`icp canister snapshot restore`) automatically if the post-deploy smoke test fails — no
manual step needed for a bad release caught by smoke. Snapshots are pruned to the last 3
per canister after a successful release. To roll back by hand (e.g. a regression found
after smoke passed):

```bash
icp canister snapshot list <canister> -e production --identity <identity>
icp canister stop <canister> -e production --identity <identity>
icp canister snapshot restore <canister> <snapshot_id> -e production --identity <identity>
icp canister start <canister> -e production --identity <identity>
```

Every release (success, rollback, abort, or failed rollback) is appended to
`docs/ops/releases.md`. A failed restore exits non-zero and is recorded as `FAILED`: treat it as
an incident and restore by hand as above. Canister ids come from `.icp/cache/mappings/local.ids.json`
on local and `.icp/data/mappings/<env>.ids.json` (committed) on staging/production; the script
fails hard when they are missing.
