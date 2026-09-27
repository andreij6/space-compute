# Lessons (active, ≤ 40 lines)

Format: `L-NNN [area] trigger → rule (source) hits:N`. Promoted lessons are removed and replaced with a pointer.

- L-001 [tooling] (promoted: the Drive connector can only create files, not edit cells; update the owner's existing sheet via Claude in Chrome: `just gantt-paste` → Name box A2:K200 → Delete → A2 → Cmd+V. Don't create a new sheet or rename theirs) Transcribing large base64 or other payloads into a tool call corrupts them and hits the output limit (planning: 5 failed Drive uploads) → never hand-copy more than 2 KB of opaque data. Use a tool that takes a file path, or hand the owner the file. (planning 2026-09-27) hits:5
- L-002 [tooling] Google Drive xlsx→Sheets conversion rejects MINIFS/MAXIFS/SUMPRODUCT/DataBar and files re-saved through an openpyxl load/save → use SUMIF/COUNTIFS and static dates; generate fresh. (planning) hits:2
- L-003 [scripting] Python inside a bash heredoc breaks on nested quotes/escapes → write the script to a file (or use a `<<'EOF'` quoted heredoc with only simple strings). (planning) hits:2
- L-004 [scripting] Schedulers over a task list KeyError when deps are defined later → topologically order first (tools/plan/plan.py does). (planning) hits:1
- L-005 [edit] "File modified since read" → re-read just the region, then edit; don't rewrite whole files. (planning) hits:2
- L-006 [owner] Anything that needs an interactive login (`/design-login`, II on mainnet, Firebase console) can't be done headless → put it on the owner's list at task start, not the end. (planning) hits:1
- L-007 [tests] PocketIC suites "pass" by skipping when the wasm or binary is missing (proof-of-burn) → report counts; a skip is a failure. hits:0
- L-008 [icp] Always pass `--identity`; an `icp canister status` without it used the password-protected `prod-deployer`, hung on a hidden password prompt and held icp's identity lock so every later icp call blocked (T1.8, 10 min lost) → promoted: guard.sh blocks icp canister/deploy/token/cycles without --identity. hits:1
- L-009 [copy] Money/threshold numbers in prose go stale (proof-of-burn) → render from config queries; after a money change, grep UI, the agent skill and docs for the old value. hits:0
- L-010 [context] Reading a whole big file to find one thing burns tokens → grep for the symbol, then read ±40 lines. hits:0
- L-011 [deploy] Ad-hoc deployment breaks canister wiring and ledger states (proof-of-burn) → always run scripts/deploy-local.sh (or just deploy-local); ledgers install once with Init, never upgrade. hits:0
- L-012 [ci] No remote CI → enforce all fmt, clippy, unit/integration, and did drift checks locally via scripts/verify-local.sh (or just verify). hits:0
- L-013 [harness] Scriptable or repetitive workflows → author a dedicated script in scripts/ and a skill in .claude/skills/ to continuously expand the harness. hits:0
- L-014 [git] Branching and PRs burn tokens on context switches and merge conflict resolution → push each completed task straight to main in a single clean commit. hits:0
- L-015 [rust] Parallel agent collisions on monolithic files and shared state → enforce agent-* rules (contract-first, crate boundaries, micro-modules < 500 LOC, hexagonal pure logic, CallerGuard). hits:0
- L-016 [code] Do not write code comments → keep code concise, clean, and self-documenting; avoid comment clutter. hits:0
- L-017 [okf] Maintain the OKF (Operational Knowledge Framework) → update okf/runbooks and docs/OKR.md on every operational/scope milestone. hits:0
- L-014 [tooling] Local `icp` was 0.2.3 while templates targeted 1.6 (schema v1.3, rust recipe v3.4.0, ic-cdk 0.20) → check `icp --version` and `cargo search` before scaffolding or adding deps; pin what the current template uses. (T1.1) hits:1
- L-015 [demo] Raw `cargo test --workspace` output buries the demo under empty crates → `just demo` runs `scripts/demo.sh`, which filters the noise and fails when zero tests match. (T1.1) hits:1
- L-016 [icp] The global default icp identity on this machine is `prod-deployer` (proof-of-burn mainnet); fresh identities have 0 cycles on the local network → always `--identity sc-*`; fund from `anonymous` (seeded). (T1.3) hits:1
- L-017 [research] research-unknowns.md claimed "CMC rejects ICRC-2 blocks" with "High" confidence; the real CMC in PocketIC accepts them → treat desk research as a hypothesis; any spike that PocketIC can run gets settled by a test, not a citation. (SP-1, SP-3) hits:2 → promoted: spikes are closed only by a `sp_*` test (see /task)
- L-018 [plan] tools/plan/plan.py schedules greedily in list order; appending a task at the end pushed launch 7 days (T1.7) → insert new tasks next to their phase peers, then check the printed milestones. hits:1
- L-019 [data] Long data jobs: split cutting (network, cached to ~/.cache) from assembly (cheap, re-runnable) so a visual fix (the rgb stretch was too dark) costs minutes, not a re-download. Calibrate on a 6-subject preview before the full run. (T1.7) hits:1
- L-020 [git] Two sessions in one checkout: my `git add -A` committed the other session's unfinished frontend under my SP commits → when another agent is active, stage explicit paths (or have each lane use its own worktree). hits:1
- L-021 [review] A parallel agent's "done" claim is a hypothesis: check each task's acceptance line in tasks.json against the code (auth wired? canister calls? tests?) before counting it. (T6.1) hits:1
- L-022 [coverage] llvm-cov only sees native unit tests, so endpoint code (msg_caller, timers, management calls) reads as uncovered (platform 38.9%) → keep canister glue in lib.rs/api.rs/timers.rs (excluded by scripts/coverage.py, PocketIC-covered) and put logic in plain modules with unit tests. (T2.1) hits:1
- L-023 [pocketic] IcpEnv.update/query encode ONE argument; a tuple becomes a record and multi-arg methods trap with "No more values on the wire" → use pic.*_call with candid::encode_args for multi-arg methods. (T2.1) hits:1
