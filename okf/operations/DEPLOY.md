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

# Or via Makefile
make deploy-local
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

# Or via Makefile
make verify
```

- Disallow test skips from masking failures (`L-007`).
- Verify AAA wasm size budget $\le 1.5$ MiB gz (`L-012`).

---

## 3. Production Deployment Protocol

Mainnet deployments are performed strictly via `icp deploy -e production` using named deployment identities with hardware wallet protection.

```bash
# 1. Switch to hardware deployment identity
icp identity use deploy-mainnet

# 2. Confirm principal
icp identity get-principal

# 3. Verify treasury and core canister builds
icp build -e production

# 4. Deploy core canisters
icp deploy treasury payments platform frontend -e production --yes

# 5. Add backup controllers immediately
icp canister settings update treasury --add-controller <BACKUP_KEY> -e production
icp canister settings update payments --add-controller <BACKUP_KEY> --add-controller <TREASURY_ID> -e production
icp canister settings update platform --add-controller <BACKUP_KEY> --add-controller <TREASURY_ID> -e production

# 6. Fund the treasury ICP float (10–20 ICP)
icp ledger transfer --to <TREASURY_ACCOUNT> --amount 10.0 -e production
```

---

## 4. Rollback & Disaster Recovery

There is no automatic rollback on ICP. If a broken wasm is deployed:
1. Keep the previous wasm binary: `target/wasm32-unknown-unknown/release/<canister>_vN-1.wasm`.
2. Reinstall/upgrade in-place:
```bash
icp canister install <canister> \
  --mode upgrade \
  --wasm target/wasm32-unknown-unknown/release/<canister>_vN-1.wasm \
  -e production
```
3. Run post-deploy smoke tests against `health()` and `get_config()`.
