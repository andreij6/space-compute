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

## Key Rules & Learnings

1. **Dedicated local identities only.** The script creates plaintext identities `sc-deployer` (controller), `sc-admin` (canister admin) and `sc-user` (test user), and passes `--identity` on every command. The machine's global default identity may be a mainnet key (e.g. `prod-deployer`) — never rely on it.
2. **New identities start empty.** The managed network seeds only the `anonymous` account. The script funds `sc-deployer` with 100T cycles (`icp cycles transfer … --identity anonymous`) and gives `sc-admin`/`sc-user` test ICP when they drop below 50.
3. **No hand-wiring of canister IDs.** `icp deploy` injects `PUBLIC_CANISTER_ID:<name>` into every canister; canisters read it at runtime. Never add `admin_set_*_canister` setters or bake IDs into code/init args.
4. **Random gateway port.** `icp.yaml` sets `gateway.port: 0`; read the URL from `icp network status -e local --json` (`gateway_url`), never assume `localhost:8000`.
5. **The ICP ledger and CMC are system canisters** on the managed network — don't deploy them yourself. ckBTC/ckETH test ledgers are added with the payments tasks (install once, never upgrade).
6. **`aaa` is built but not deployed** in any environment; the platform stores its wasm (from T2.2).
7. **Verify before committing:** `just verify` (fmt, clippy, candid drift, aaa size, tests with skip=fail, no dfx). `just demo T1.3` proves the deploy is idempotent.
