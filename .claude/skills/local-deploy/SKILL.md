---
name: local-deploy
description: "Instructions, commands, and rules for deterministic, repeatable local deployments of Space Compute using icp and scripts/deploy-local.sh. Covers local managed network start, one-time ledger installation, core canister deployments, inter-canister wiring, and mock data seeding. Use whenever deploying, updating, or testing the app locally, or recovering from network wipes."
license: Apache-2.0
metadata:
  title: Local Deploy & Repeatable Environments
  category: DevOps
---

# Local Deploy & Repeatable Environments

## Core Philosophy
Local deployment must be completely **deterministic, repeatable, and idempotent**. Any agent or developer should be able to run a single command and have the entire system—network, token ledgers, core canisters, templates, and initial seed data—in a clean, running, wired state.

## Primary Commands

```bash
# Recommended one-shot command
bash scripts/deploy-local.sh

# Or via Justfile
just deploy-local
```

## Key Rules & Learnings (from Proof of Burn & Space Compute)

1. **Ledgers are Installed Once with `Init`, Never Upgraded:**
   - ICRC-1 / ICRC-2 ledgers (`ledger`, `ckbtc-ledger`, `cketh-ledger`) take an `Init` variant argument upon creation.
   - Running `icp deploy` or `icp canister install --mode upgrade` on an already installed ledger traps with `"Cannot upgrade ... Init argument"`.
   - The deploy script guards ledger deployments by testing `canister_exists <name>` first.

2. **Canister IDs Permute on Network Wipe:**
   - When the local network is reset, canisters are assigned IDs in creation order.
   - Never bake hard-coded canister principals into Rust source code.
   - The deploy script captures live canister IDs in `.env.local/canister_ids.env` and uses admin calls (`admin_set_*`) to wire inter-canister principals dynamically.

3. **Always Pass `--identity` and `-e local`:**
   - To avoid default identity pollution, `scripts/deploy-local.sh` uses explicit dev identities:
     - `DEPLOY_IDENTITY`: `dev-deployer` (controls canisters)
     - `ADMIN_IDENTITY`: `dev1` (canister admin for configuration and wiring)
     - `TEST_IDENTITY`: `dev2` (test agent/user)

4. **Verify Canister Wiring Post-Deploy:**
   - After deploying `platform`, `payments`, and `treasury`, verify that:
     - `payments` knows `platform` canister ID.
     - `treasury` watches `platform` and `payments` canister cycles.
     - Local dev faucet transfers test ICP to `ADMIN_IDENTITY` for spawn testing.

5. **Local Verification Before Merge:**
   - Run `just verify` (or `bash scripts/verify-local.sh`) to run `cargo fmt`, `cargo clippy`, and unit/integration tests locally before marking tasks complete.
