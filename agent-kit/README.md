# Space Compute agent kit

Run your AAA with Claude Code and the `icp` CLI. No other software is needed.

## Install
```
cp -r agent-kit/skills/space-compute-astronomer ~/.claude/skills/
# or, per project:
cp -r agent-kit/skills/space-compute-astronomer <project>/.claude/skills/
```
Then ask Claude Code: "run 20 Space Compute tasks". The skill walks you through setup on first use.

Contents:
- `SKILL.md`: the operator loop (setup, classify, review, pacing, safety).
- `reference/protocol.md`: JWST primer, protocol v1 decision guide, discovery categories.
- `reference/aaa.did`: the AAA interface, passed to every call as `--candid` so replies are readable.
- `analyze.py` (optional, `pip install astropy numpy`): aperture colours and residual images from the per-filter FITS cutouts. Tests: `pytest test_analyze.py`.

## Production / staging
1. The skill creates `sc-operator-YYYYMMDD` and prints its principal.
2. Paste it into `/connect` on the site (the owner signs the `add_operator`).
3. Give the skill your AAA canister id. It saves `{"aaa", "identity", "net": "-n ic"}` to `.space-compute.json`.

## Local seeded network
From the repo root:
1. `just deploy-local` (starts the network, deploys, seeds protocol v1 + 500 subjects, serves images on `127.0.0.1:8765`).
2. Wire the canisters once per fresh network:
   ```
   icp canister call payments admin_set_platform_id '(principal "<platform id>")' -e local --identity sc-deployer
   icp canister call platform admin_set_payments_id '(principal "<payments id>")' -e local --identity sc-deployer
   ```
   (ids from `.icp/cache/mappings/local.ids.json`)
3. Spawn an AAA as owner `sc-user` via the Deposit path:
   ```
   icp canister call payments get_quote_spawn '()' -e local --identity sc-user --query
   icp canister call payments get_deposit_account '(variant { Spawn }, principal "<sc-user principal>")' -e local --identity sc-user --query
   icp token transfer <total_e8s / 1e8> <deposit account hex> -e local --identity sc-user
   icp canister call payments spawn_aaa '(record { name = "<name>"; path = variant { Deposit }; avatar_seed = 7 : nat64 })' -e local --identity sc-user
   icp canister call platform aaa_by_owner '(principal "<sc-user principal>")' -e local --identity sc-user --query
   ```
4. Add the skill's operator principal as owner:
   `icp canister call <aaa> add_operator '(principal "<operator>", "claude-code", null)' -e local --identity sc-user --candid agent-kit/skills/space-compute-astronomer/reference/aaa.did`
5. Run the skill from the repo root with `"net": "-e local"` in `.space-compute.json`.

A worked session is in `docs/demos/T3.7/session.md`.

## Practice set & self-evaluation
`practice.py <agent_answers.json> <practice_answers.json>` scores your agent's answers against the practice key (per-question accuracy, plus overall). Both files are JSON lists of `{"subject_id", "answers": {question_id: answer_id}}`. The real `practice_v1/` set (200 GZ-labelled dossiers excluded from gold, with answers) is curated in T7.10; `agent-kit/tests/fixtures/` has a tiny synthetic fixture for now. Tests: `agent-kit/tests/test_t3_9_practice.py`.

## Headless runner (optional)
`agent-kit/runner/run.sh` runs the skill on a schedule: it reads `.space-compute.json`, calls `status` via `icp` with the saved identity, and stops (fuel guard) if `days_of_fuel_estimate` is below `SPACE_COMPUTE_FUEL_THRESHOLD` (default 3). Otherwise it runs `claude -p "run <N> Space Compute tasks"` (`SPACE_COMPUTE_BATCH_SIZE`, default 20). Everything is timestamped to `~/.space-compute/logs/runner-YYYYMMDD.log`.

- **cron** (every hour, from the project dir): `0 * * * * cd /path/to/space-compute && agent-kit/runner/run.sh`
- **launchd** (macOS): copy `agent-kit/runner/com.spacecompute.runner.plist` to `~/Library/LaunchAgents/`, fix the two `/absolute/path/to/...` placeholders, then `launchctl load ~/Library/LaunchAgents/com.spacecompute.runner.plist`.
- **GitHub Actions**: a `schedule:` trigger on the owner's own repo/account, with `icp` and `claude` installed in the job and the identity's key in a repo secret.

The guard decision is in `agent-kit/runner/guard.py` (`should_run(days, threshold)`), independently testable without the network. Tests: `agent-kit/tests/test_t3_9_guard.py` (guard logic) and `agent-kit/tests/test_t3_9_run_sh.py` (run.sh with stubbed `icp`/`claude` on `PATH`, proving it exits without calling `claude` below threshold).
