# Deploy runbook — first staging deploy

Spec: `docs/specs/09-testing-ops.md` §2-3, `docs/specs/01-architecture.md` §4b/§5/§5b, `docs/specs/12-treasury-keeper.md`.

Proven end to end on `local` only (`scripts/deploy-env.sh local` run twice, a forced smoke
failure + rollback, `scripts/cycles-report.sh local`, `scripts/treasury-watch.sh local`).
Nothing here has touched staging or mainnet — `.claude/harness/hooks/guard.sh` blocks any
`icp` command targeting `ic`/`staging`/`production` unless `SC_ALLOW_MAINNET=1`, and
`scripts/deploy-env.sh` itself refuses non-local environments unless `SC_ALLOW_MAINNET=1`
plus a typed confirmation.

## 1. Before the first staging deploy

1. Create a staging deploy identity (never the machine default `prod-deployer`):
   ```bash
   icp identity new sc-staging-deployer --storage plaintext
   ```
2. Fund it with cycles (staging canisters are created with real cycles, not test ones — the
   owner funds this out of band; see `docs/specs/12-treasury-keeper.md` §4 for the ICP->CMC
   path once `treasury` is deployed).
3. Confirm `icp.yaml`'s `staging` environment (network `ic`, canisters `platform payments
   treasury frontend`) is what you want to create — canister IDs are permanent once created.

## 2. Run the release

```bash
export SC_ALLOW_MAINNET=1
export DEPLOY_IDENTITY=sc-staging-deployer
just release staging
# or: bash scripts/deploy-env.sh staging
```

You will be prompted to type `staging` to confirm. The script then:
1. Preflight: `just verify` green, clean git tree, on `main`, identity explicit.
2. Snapshots every canister that already exists (skipped on first install).
3. `icp deploy -e staging --identity sc-staging-deployer --yes`.
4. Wires `platform.payments_id` <-> `payments.platform_id`.
5. Registers/approves the AAA template wasm if `AAA_REGISTER=1` is set (slow reproducible
   build — see `docs/ops/aaa-manual-upgrade.md`).
6. Smoke test: `version()` on every canister, `platform.get_stats()`, `treasury.health()`,
   and an HTTP GET on the frontend.
7. On any failure: restores every snapshot taken in step 2 and exits non-zero.
8. On success: prunes snapshots to the last 3 per canister and appends a row to
   `docs/ops/releases.md`.

## 3. Right after the first successful staging deploy

1. Make treasury a co-controller and populate its watch list:
   ```bash
   DEPLOY_IDENTITY=sc-staging-deployer just watch-treasury staging
   ```
2. Add the backup/team controllers per `docs/specs/01-architecture.md` §5b and
   `okf/operations/OPS.md` §2 (treasury is added by step 1; the human backup principal is
   added by hand):
   ```bash
   icp canister settings update platform --add-controller <BACKUP_PRINCIPAL> -e staging --identity sc-staging-deployer
   icp canister settings update payments --add-controller <BACKUP_PRINCIPAL> -e staging --identity sc-staging-deployer
   icp canister settings update frontend --add-controller <BACKUP_PRINCIPAL> -e staging --identity sc-staging-deployer
   ```
3. Set freezing thresholds per `icp.yaml`/01 §5b if not already applied by the recipe.
4. Fund the treasury float (10-20 ICP) once the owner is ready:
   ```bash
   icp ledger transfer --to <treasury-canister-id> --amount 15.0 -e staging --identity sc-staging-deployer
   ```
5. Set up cycles monitoring (pick one):
   - launchd (macOS): copy `scripts/com.spacecompute.cycles-report.plist`, fix the script
     path and `DEPLOY_IDENTITY`, then
     `launchctl load ~/Library/LaunchAgents/com.spacecompute.cycles-report.plist`.
   - cron: `0 */6 * * * DEPLOY_IDENTITY=sc-staging-deployer /path/to/scripts/cycles-report.sh staging >> /tmp/sc-cycles-report.log 2>&1`
   Either way it exits non-zero when a canister drops under `CYCLES_ALERT_DAYS` (default 30)
   or `treasury.health().reserve_breached` is true.

## 4. What the owner must do (only the owner should do these)

- Type `staging` at the confirmation prompt and hold `DEPLOY_IDENTITY`/`SC_ALLOW_MAINNET=1`
  for the session — never bake `SC_ALLOW_MAINNET=1` into a shell profile.
- Provide the backup controller principal(s) for step 3.2.
- Authorize and perform the treasury ICP funding transfer (step 3.4) from the owner wallet.
- Decide when to promote the same workflow (`scripts/deploy-env.sh production`) to mainnet,
  after a staging soak per `docs/specs/09-testing-ops.md` §3 ("AAA wasm version is bumped in
  `platform` only after staging soak >= 48h").

## 5. Rollback

Automatic on smoke failure (see step 2.7). To roll back manually after the fact:

```bash
icp canister snapshot list <canister> -e staging --identity sc-staging-deployer
icp canister stop <canister> -e staging --identity sc-staging-deployer
icp canister snapshot restore <canister> <snapshot_id> -e staging --identity sc-staging-deployer
icp canister start <canister> -e staging --identity sc-staging-deployer
```
