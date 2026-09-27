"""Single source for the task plan.
Usage: python tools/plan/plan.py  (needs openpyxl)
Emits docs/specs/10-tasks.md, docs/space-compute-gantt-v3.xlsx and .claude/harness/tasks.json.
Status/pct live in tasks.json (the harness edits them); they are merged back on every run."""
import datetime as dt, sys, json, os
from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.formatting.rule import FormulaRule, DataBarRule
from openpyxl.chart import BarChart, Reference
from openpyxl.utils import get_column_letter as L
from openpyxl.worksheet.datavalidation import DataValidation

START = dt.date(2026, 9, 28)
HOLIDAYS = {dt.date(2026, 11, 26), dt.date(2026, 11, 27), dt.date(2026, 12, 24), dt.date(2026, 12, 25),
            dt.date(2026, 12, 31), dt.date(2027, 1, 1)}

PHASES = {
 "P0": ("Planning", "B4B4B4"), "P1": ("Foundations & spikes", "8AB4F8"), "P2": ("Platform core", "81C995"),
 "P3": ("AAA canister & agent kit", "FDD663"), "P4": ("Review, consensus & credits", "F28B82"),
 "P5": ("Payments", "C58AF9"), "P6": ("Frontend", "78D9EC"), "P7": ("Hardening", "FCAD70"),
 "P8": ("Beta & launch", "A8DAB5"), "P9": ("Design & polish (after function)", "F6AEA9"),
}
# id, phase, name, (legacy role, ignored), agent-days, deps, spec, acceptance, pct
# Claude Code does all coding; tier "-" tasks are the human's (sign-off, beta, neuron, accounts, external audit).
T = [
 ("T0.1","P0","Overview & design brief","Owner",3,[],"OVERVIEW, DESIGN_BRIEF","Approved docs",1.0),
 ("T0.2","P0","Detailed specs + adversarial review","Owner",3,["T0.1"],"specs/01-09, REVIEW","All findings resolved or spiked",1.0),
 ("T0.3","P0","Gantt, review deck, OKRs","Owner",1,["T0.2"],"OKR.md","Artifacts published",1.0),
 ("T0.4","P0","Owner review & sign-off of plan","Owner",3,["T0.3"],"—","Go / no-go recorded in OKR.md",0.0),
 ("T1.1","P1","Repo scaffold: cargo workspace, icp.yaml, toolchain, .gitignore","BE-A",1,["T0.4"],"01 §3","`icp build` succeeds for empty canisters",0),
 ("T1.2","P1","sc-types shared crate (types, ApiError, limits)","BE-A",2,["T1.1"],"02 §2, 08 §3","Types compile; candid round-trip tests",0),
 ("T1.3","P1","Local verification & deploy harness (verify-local.sh, deploy-local.sh, did drift, wasm size)","BE-A",2,["T1.1"],"09 §2","Local verification & deploy scripts pass",0),
 ("T1.4","P1","PocketIC harness with ICP ledger + CMC wasms","BE-B",2,["T1.1"],"09 §1","Harness test mints ICP, calls CMC",0),
 ("SP-7","P1","Spike: Galaxy Zoo JWST (CEERS) labels, licence, mapping","BE-B",1,["T0.4"],"REVIEW SP-7","Gold source decided",0),
 ("T1.5","P1","JWST subject selection + gold (DJA v7 catalogs, 6 fields)","BE-B",5,["SP-7"],"07 §1,§4,§5.1-2","20k selected; ≥2k gold",0),
 ("T1.7","P1","Render dossiers (cutouts, RGB, dossier.json) + R2 bucket + manifest","BE-B",5,["T1.5"],"07 §3,§5.3-5","Hash verify; 40 QA spot-checks",0),
 ("SP-1","P1","Spike: CMC + ICRC-2 memo behaviour","BE-B",1,["T1.4"],"REVIEW SP-1","Decision recorded in 04 §2",0),
 ("SP-3","P1","Spike: frozen canister query behaviour","BE-B",1,["T1.4"],"REVIEW SP-3","Decision recorded",0),
 ("SP-4","P1","Spike: canister_info cost/latency","BE-A",1,["T1.4"],"REVIEW SP-4","Decision recorded",0),
 ("SP-6","P1","Spike: ckETH helper subaccount deposit + minimum","BE-B",1,["T1.4"],"REVIEW SP-6","Decision recorded in 04 §6.6",0),
 ("SP-2","P1","Spike: OISY approve w/ spender subaccount","FE",1,["T0.4"],"REVIEW SP-2","Decision recorded",0),
 ("T1.8","P1","Test infra: coverage gate, proptest, fuzz, bot-agent harness, traceability, fixture dossiers (Playwright+axe → T6.10)","BE-A",3,["T1.3","T1.4"],"11","Gates run in just verify on empty suites",0),
 ("T2.1","P2","platform skeleton: config/admin, memory map, timers, RNG","BE-A",2,["T1.2"],"02 §3, §9-10","Upgrade keeps config",0),
 ("T2.2","P2","Registry & factory: register/install/verify/upgrade/profile","BE-A",4,["T2.1","SP-4"],"02 §4","Spawned AAA verified; upgrade works",0),
 ("T2.3","P2","Catalog: subjects, protocol, leases, seen-set, get_task (gold+calibration)","BE-A",4,["T2.1"],"02 §5.1","Never same subject twice",0),
 ("T2.4","P2","Scoring: validation, gold, tallies, retirement, consensus","BE-A",4,["T2.3"],"02 §5.2, §5.5","Retire at K=5; idempotent submit",0),
 ("T2.5","P2","Event log + per-AAA index + activity query","BE-A",2,["T2.1"],"02 §8.1","Events append; paged activity",0),
 ("T2.6","P2","Public queries: stats, protocol, aaa_by_owner, aaa_public","BE-A",1,["T2.2"],"02 §7","Queries paged ≤100",0),
 ("T2.8","P2","Submitter security: strict provenance, operator sync, submitted_by","BE-A",3,["T2.2"],"02 §4.6, §5; 08 S21-23","Foreign/expired/unsynced submitter rejected",0),
 ("T2.7","P2","Platform PocketIC tests (02 §11 #1-3)","BE-A",2,["T2.4","T1.4"],"02 §11","Tests green",0),
 ("T3.1","P3","AAA skeleton: roles, operators, config, inspect_message","BE-B",2,["T1.2"],"03 §1-3","Role matrix enforced",0),
 ("T3.2","P3","AAA forwarding w/ fees, retries, idempotency, low-cycles guard","BE-B",3,["T3.1","T2.4"],"03 §4.1","One record per task under SYS_UNKNOWN",0),
 ("T3.3","P3","AAA repository records + queries + credits copy","BE-B",2,["T3.2"],"03 §5","Records survive upgrade",0),
 ("T3.4","P3","AAA timers: burn EMA, heartbeat, credits sync, auto top-up trigger","BE-B",2,["T3.3"],"03 §6","Timer tests pass",0),
 ("T3.5","P3","get_api_doc + wasm size budget ≤1.5 MiB gz","BE-B",1,["T3.2"],"03 §4.3","Size check in just verify",0),
 ("T3.6","P3","AAA PocketIC tests (03 §8)","BE-B",2,["T3.4"],"03 §8","Tests green",0),
 ("T3.8","P3","Local seed script + dev loop (tools/seed-local)","BE-B",1,["T2.2","T3.2","T1.7","T1.8"],"05 §5","One command seeds local net",0),
 ("T3.7","P3","Operator skill + JWST protocol primer + analyze.py","BE-B",4,["T3.5","T1.7"],"06","10 local tasks by Claude Code",0),
 ("T3.9","P3","Headless runner recipe + practice.py self-eval","BE-B",1,["T3.7"],"06 §2b-2c","Runner stops at fuel guard",0),
 ("T4.3","P4","Progression: XP, reputation, tiers, badges, leaderboard, replay","BE-A",4,["T2.5","T2.7"],"02 §8.2","Replay == incremental",0),
 ("T4.1","P4","Discoveries: flagging, rate limit, public IDs","BE-A",2,["T2.4"],"02 §6","Flag rate enforced",0),
 ("T4.9","P4","First-claim rule: sky claim index + corroborations","BE-A",2,["T4.1"],"02 §6.4","Concurrent flags → one discovery",0),
 ("T4.2","P4","Review assignment: eligibility, blind, leases, honeypots","BE-A",3,["T4.1","T4.3"],"02 §5.3, §6.3","Same-owner never assigned",0),
 ("T4.4","P4","evaluate(), atomic resolution, starvation rule","BE-A",3,["T4.2"],"02 §6.2","Atomic resolution test",0),
 ("T4.5","P4","Citations + certification tree + get_citation","BE-A",3,["T4.4"],"02 §8.3","Witness verifies after upgrade",0),
 ("T4.6","P4","Visibility rules & public discovery queries","BE-A",2,["T4.5"],"02 §7","Under-review hidden from non-owners",0),
 ("T4.7","P4","Honeypot seeding (120 specs)","BE-B",1,["T4.2","T1.7"],"07 §3.6","Honeypots uploaded locally",0),
 ("T4.10","P4","Admin read APIs + audit log (platform)","BE-A",2,["T4.6"],"02 §9","Admin queries paged; audit on every mutation",0),
 ("T4.11","P4","Name blocklist, admin_rename_aaa, house AAAs","BE-A",1,["T4.10"],"02 §9","Renamed AAA keeps historical citation name",0),
 ("T4.8","P4","Review/credits tests + proptests (02 §11 #4-9)","BE-A",3,["T4.6","T4.9"],"02 §11, 09 §1","Tests green",0),
 ("T5.1","P5","payments skeleton: config, journal, guards, admin","BE-B",2,["T1.2","SP-1"],"04 §3","Journal persisted pre-await",0),
 ("T5.2","P5","CMC integration, XDR cache, quotes, deposit accounts","BE-B",2,["T5.1"],"04 §2, §4","Quotes within 2%",0),
 ("T5.3","P5","Spawn saga (wallet + deposit) + resume timer","BE-B",4,["T5.2","T2.2"],"04 §4","04 §5 #1,#2,#4",0),
 ("T5.4","P5","One-time top-up (wallet + deposit sweep)","BE-B",2,["T5.2"],"04 §4","Gift top-up works",0),
 ("T5.5","P5","Auto top-up mandates, rolling 30-day cap","BE-B",3,["T5.4","T3.4"],"04 §4","04 §5 #5",0),
 ("T5.6","P5","Payments PocketIC tests — ICP paths (04 §5)","BE-B",3,["T5.3","T5.5"],"04 §5","All 7 green",0),
 ("T5.8","P5","XRC rates, fuel treasury guard rails + auto intake pause","BE-B",4,["T5.2"],"04 §6.1-6.3, §6.2b","04 §6.7 #11-13, #15-17",0),
 ("T5.12","P5","stripe_credit endpoint (flag-gated, mock-relay tests)","BE-B",2,["T5.8"],"04 §6.4, 04b §3","Idempotent on stripe_ref",0),
 ("T5.9","P5","ckBTC fuel-pack deposits (address, update_balance, sweep)","BE-B",3,["T5.8"],"04 §6.5","04 §6.7 #10",0),
 ("T5.10","P5","ckETH fuel-pack deposits","BE-B",2,["T5.8","SP-6"],"04 §6.6","Mock mint → top-up",0),
 ("T5.13","P5","Non-ICP payment tests (04 §6.7)","BE-B",2,["T5.9","T5.10","T5.12"],"04 §6.7","All green",0),
 ("T5.15","P5","Payments admin APIs + audit log","BE-B",1,["T5.13"],"04 §4 admin","Overview matches ledger balances",0),
 ("T5.16","P5","Feature flags + invite codes (sponsored spawn)","BE-B",2,["T5.3"],"04 §0-0b","card=false → FeatureDisabled; invite single-use",0),
 ("T5.17","P5","treasury canister: owner-funded reserve, cycles keeper, health()","BE-A",3,["T2.1","T1.4"],"12 §1-4","Keeper tops up; health() drives intake pause",0),
 ("T5.18","P5","treasury PocketIC tests (ICP ledger + CMC, no NNS)","BE-A",1,["T5.17"],"12 §5","12 §5 #1-6 green",0),
 ("T5.7","P5","Internal payments security review","BE-A",2,["T5.6","T5.13","T5.16"],"08 §4","Checklist signed",0),
 ("T6.1","P6","App scaffold: routing, bindgen, II auth, ic_env, CSP","FE",3,["T1.1"],"05 §1","Sign-in works locally",0),
 ("T6.3","P6","Landing, Discovery Museum feed, Discovery detail + citation verify","FE",5,["T6.1"],"05 §2-3","Verified mark fails closed",0),
 ("T6.4","P6","AAA profile + Leaderboard","FE",3,["T6.3"],"05 §2","Tier/badges render",0),
 ("T6.5","P6","Payment component (wallet + deposit)","FE",4,["T6.1","SP-2","T5.4"],"05 §3","Deposit path e2e local",0),
 ("T6.6","P6","Spawn flow","FE",2,["T6.5","T5.3"],"05 §2","Spawn e2e local",0),
 ("T6.7","P6","Owner dashboard","FE",4,["T6.1","T3.3","T4.6"],"05 §2-3","Frozen fallback works",0),
 ("T6.8","P6","Connect your agent + Activity & records","FE",3,["T6.7"],"05 §3","Operator add/revoke",0),
 ("T6.9","P6","Fuel & billing + auto top-up","FE",3,["T6.5","T5.5"],"05 §3","Mandate states render",0),
 ("T6.11","P6","BTC & ETH methods + feature-flag gating (card hidden)","FE",4,["T6.5","T5.9","T5.10"],"05 §3","Test-mode card + mock BTC/ETH e2e",0),
 ("T6.12","P6","Admin console (8 screens)","FE",5,["T6.1","T4.10","T5.15"],"05 §2b","Non-admin gets 404/Unauthorized",0),
 ("T6.13","P6","Firebase analytics + consent + event taxonomy + perf traces","FE",2,["T6.1"],"05 §4b","No requests before consent",0),
 ("T6.14","P6","About/legal/practice pages, invite spawn UI, admin invites/treasury/moderation","FE",3,["T6.12","T5.16","T5.18"],"05 §2, §2b","E2E: invite spawn; non-admin blocked",0),
 ("T6.10","P6","States, accessibility, Playwright smoke","FE",3,["T6.4","T6.6","T6.8","T6.9","T6.11","T6.12","T6.13","T6.14"],"05 §5","Lighthouse a11y ≥90",0),
 ("T7.1","P7","Upgrade tests for all canisters","BE-A",2,["T4.8","T5.6"],"09 §1","vN-1→vN state intact",0),
 ("T7.2","P7","Load test: 200 AAAs × 100 tasks","BE-A",2,["T7.1"],"09 §1","Report committed",0),
 ("T7.3","P7","Cycle cost measurement & fee retune","BE-A",2,["T7.2"],"01 §7, SP-5","Params updated",0),
 ("T7.4","P7","Reproducible AAA build + manual upgrade docs","BE-B",2,["T3.6","T5.7"],"08 §7","Hash reproducible on 2 machines",0),
 ("T7.7","P7","Prompt-injection test with reference agent","BE-B",1,["T3.7","T4.7"],"06 §5","≥95% non-compliance",0),
 ("T7.9","P7","Traceability audit + coverage gates to thresholds","BE-A",2,["T7.1"],"11 §2","Every acceptance item mapped; coverage ≥ gates",0),
 ("T7.10","P7","Practice set + open data release tooling","BE-B",2,["T1.7","T4.8"],"07 §5b","Release v0 reproducible",0),
 ("T7.5","P7","External security review (payments + platform)","Vendor",5,["T5.7","T4.8"],"08 §4","Report received",0),
 ("T7.6","P7","Fix security findings","BE-B",3,["T7.5"],"08","All high/critical closed",0),
 ("T8.1","P8","Staging deploy, deploy workflow, snapshots, cycles monitoring","BE-A",2,["T7.1"],"09 §2-3","Staging live",0),
 ("T8.2","P8","Upload subjects/protocol/honeypots to staging","BE-B",1,["T8.1","T7.6"],"07","Counts verified",0),
 ("T8.5","P8","Fund prod treasury with ICP float (10–20 ICP) + confirm runway","Owner",1,["T8.6"],"12 §4","status() shows ≥ 90 days runway",0),
 ("T8.3","P8","Closed beta (10-20 owners)","Owner",10,["T8.2","T6.10"],"OKR KR","Beta KRs measured",0),
 ("T8.4","P8","Beta fixes & tuning","BE-A",10,["T8.2","T6.10"],"—","No open P0/P1 bugs",0),
 ("T8.6","P8","Production deploy, II metadata, treasury watch list","BE-A",2,["T8.3","T8.4","T8.13"],"09 §2","Prod live",0),
 ("T8.7","P8","Public launch","Owner",1,["T8.6","T8.5","T9.5"],"OKR","Launch announced",0),
 ("T9.1","P9","Import mockups (Claude Design), reconcile, design tokens + shared components","FE",4,["T6.10"],"05 intro","05a-design-reconciliation.md + component page",0),
 ("T9.2","P9","Style public pages: landing, museum, discovery, profile, leaderboard","FE",4,["T9.1"],"05 §2-3, mockups","Visual snapshots approved by owner",0),
 ("T9.3","P9","Style owner flows: spawn, dashboard, connect agent, fuel, payments","FE",4,["T9.1"],"05 §2-3, mockups","Visual snapshots approved by owner",0),
 ("T9.4","P9","Style admin console + about/legal/practice pages","FE",2,["T9.1"],"05 §2b","Visual snapshots",0),
 ("T9.5","P9","Responsive/mobile pass, visual-regression baseline, a11y + Lighthouse re-run","FE",2,["T9.2","T9.3","T9.4"],"05 §4-5","Lighthouse a11y ≥90, perf ≥80; baseline committed",0),
 ("T8.13","P8","Custom domain + II alternative origins","Owner",1,["T8.1"],"05 §1","Domain serves frontend; II principal stable",0),
 ("T8.12","P8","Firebase projects (staging/prod) + GA4 funnels & dashboards","Owner",1,["T0.4"],"05 §4b","Funnel dashboard live",0),
 ("T8.8","P8","Compliance check for card & crypto fuel packs (recommended)","Owner",5,["T0.4"],"08 S20","Written go/no-go",0),
 ("T8.9","P8","Enable BTC/ETH on production (card stays off)","Owner",1,["T8.6","T8.8"],"04 §6.2","admin_pause_non_icp off",0),
]
MILESTONES = [("M1 Foundations + JWST data ready","T1.7"),("M2 Local alpha: agent classifies end-to-end","T3.8"),
 ("M3 Review & credits complete","T4.8"),("M4 Payments complete","T5.7"),("M5 Frontend feature-complete (unstyled)","T6.10"),("M5b Design applied","T9.5"),
 ("M6 Staging loaded & beta-ready backend","T8.2"),("M7 Public launch","T8.7")]

# Model tier = intelligence needed. H: Opus 5.5 (high effort) — novel design, security, money sagas, consensus.
# M: Sonnet 5 — well-specified feature work. L: Haiku 4.5 — mechanical (scaffold, config, queries, copy, seed).
# "-": human task. Any L/M task that touches money or auth code gets an H review before it is marked done.
TIER = dict(x.split(":") for x in """T0.1:- T0.2:- T0.3:- T0.4:- SP-1:M SP-2:M SP-3:L SP-4:L SP-6:M SP-7:M
T1.1:L T1.2:M T1.3:L T1.4:M T1.5:M T1.7:M T1.8:M
T2.1:M T2.2:H T2.3:M T2.4:H T2.5:M T2.6:L T2.7:M T2.8:H
T3.1:M T3.2:H T3.3:L T3.4:M T3.5:L T3.6:M T3.7:H T3.8:L T3.9:L
T4.1:M T4.2:H T4.3:M T4.4:H T4.5:H T4.6:M T4.7:L T4.8:M T4.9:H T4.10:L T4.11:L
T5.1:M T5.2:M T5.3:H T5.4:M T5.5:M T5.6:M T5.7:H T5.8:H T5.9:H T5.10:M T5.12:L T5.13:M T5.15:L T5.16:M T5.17:H T5.18:M
T6.1:M T6.3:M T6.4:L T6.5:M T6.6:L T6.7:M T6.8:L T6.9:L T6.10:M T6.11:M T6.12:M T6.13:L T6.14:L
T7.1:M T7.2:M T7.3:M T7.4:M T7.5:- T7.6:H T7.7:H T7.9:L T7.10:L
T8.1:M T8.2:L T8.3:- T8.4:M T8.5:- T8.6:M T8.7:- T8.8:- T8.9:- T8.12:- T8.13:-
T9.1:M T9.2:M T9.3:M T9.4:L T9.5:L""".split())
MODEL = {"H": "opus-5.5 (high)", "M": "sonnet-5", "L": "haiku-4.5", "-": "human"}
def demo(tid, ph, name):
    """What the owner looks at to accept the task (keep it < 1 minute)."""
    if TIER[tid] == "-" or tid.startswith("T0"): return "note: 3-line summary in chat"
    if tid.startswith("SP-"): return "note: decision + 1 proof command"
    if ph in ("P6", "P9"): return f"shots: docs/demos/{tid}/*.png (Playwright)"
    if tid in ("T1.5", "T1.7", "T4.7", "T7.10"): return f"image: docs/demos/{tid}/contact-sheet.png"
    return f"test: just demo {tid}  (narrated PocketIC/pytest run)"

def is_work(d): return d.weekday() < 5 and d not in HOLIDAYS
def next_work(d):
    while not is_work(d): d += dt.timedelta(1)
    return d
def add_days(start, n):  # inclusive end after n working days
    d = next_work(start); n -= 1
    while n > 0:
        d += dt.timedelta(1)
        if is_work(d): n -= 1
    return d

# LANES = parallel Claude Code sessions, each in its own git worktree. 3 because the owner can review ~3 demo
# cards a day and more lanes mostly buy merge conflicts in shared crates. Greedy list scheduling:
# each agent task takes the lane that frees up first; human tasks (tier "-") use no lane.
LANES = 3
lane_free = [START] * LANES
lane = {}
end, start = {}, {}
owner_free = {}
P0_START = dt.date(2026, 9, 21)
FORCED = {"T0.1": (dt.date(2026,9,21), dt.date(2026,9,23)), "T0.2": (dt.date(2026,9,24), dt.date(2026,9,25)),
          "T0.3": (dt.date(2026,9,26), dt.date(2026,9,26))}
_done=set(); _order=[]
while len(_order)<len(T):
    for t in T:
        if t[0] not in _done and all(d in _done for d in t[5]):
            _order.append(t); _done.add(t[0]); break
for tid, ph, name, owner, days, deps, *_ in _order:
    if tid in FORCED:
        start[tid], end[tid] = FORCED[tid]; continue
    s = START
    for d in deps: s = max(s, end[d] + dt.timedelta(1))
    if TIER[tid] != "-":
        k = min(range(LANES), key=lambda i: max(s, lane_free[i]))
        s = max(s, lane_free[k]); lane[tid] = k + 1
    s = next_work(s); e = add_days(s, days)
    start[tid], end[tid] = s, e
    if tid in lane: lane_free[lane[tid] - 1] = e + dt.timedelta(1)

def md():
    out = ["# 10 — Task plan (source for the Gantt chart)", "",
           f"Generated from the plan model; baseline start {START}. **Claude Code writes all the code** in {LANES} parallel lanes (parallel sessions committing straight to `main`; `Lane` column). Rows with lane `you` are the owner's: sign-off, beta, treasury funding, accounts, external audit. Days are agent-days, including tests, demo and review. US holidays excluded.",
           "Each task's detailed specification is the linked spec section. A task is done when its acceptance check passes in `just verify` (no remote CI) or is recorded in the named doc.", ""]
    for ph, (pname, _) in PHASES.items():
        rows = [t for t in T if t[1] == ph]
        out += [f"## {ph} — {pname}", "", "| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |", "|---|---|---|---|---|---|---|---|---|---|---|"]
        for tid, _, name, owner, days, deps, spec, acc, _p in rows:
            out.append(f"| {tid} | {name} | {TIER[tid]} | {lane.get(tid, 'you')} | {days} | {', '.join(deps) or '—'} | {start[tid]} | {end[tid]} | {spec} | {acc} | {demo(tid, ph, name)} |")
        out.append("")
    out += ["## Deferred (not scheduled — owner decision 2026-09-27: Stripe hidden at launch)", "",
            "| ID | Task | Spec |", "|---|---|---|",
            "| D1 | Stripe relay Worker: checkout, webhook, portal | 04b |",
            "| D2 | Stripe account, products, Radar | 04b §2 |",
            "| D3 | Card tab + subscription management UI | 05 §3 |",
            "| D4 | Stripe reconciliation job + enable `features.card` | 04b §3, 09 §3 |", ""]
    out += ["## Milestones", "", "| Milestone | Gate task | Date |", "|---|---|---|"]
    out += [f"| {m} | {t} | {end[t]} |" for m, t in MILESTONES]
    return "\n".join(out) + "\n"

def xlsx(path):
    wb = Workbook()
    thin = Side(style="thin", color="DDDDDD")
    hdr_font = Font(bold=True, color="FFFFFF"); hdr_fill = PatternFill("solid", fgColor="202124")
    # ---- Gantt
    ws = wb.active; ws.title = "Gantt"
    cols = ["ID","Phase","Task","Model","Start","End","Work days","Depends on","% Complete","Status","Done days"]
    first_mon = min(start.values()) - dt.timedelta(days=min(start.values()).weekday())
    last = max(end.values()); weeks = []
    d = first_mon
    while d <= last: weeks.append(d); d += dt.timedelta(7)
    ws.append(cols + weeks)
    for c in range(1, len(cols) + len(weeks) + 1):
        cell = ws.cell(1, c); cell.font = hdr_font; cell.fill = hdr_fill
        cell.alignment = Alignment(horizontal="center", vertical="center", text_rotation=90 if c > len(cols) else 0)
    for c in range(len(cols) + 1, len(cols) + len(weeks) + 1):
        ws.cell(1, c).number_format = "mmm d"; ws.column_dimensions[L(c)].width = 3.2
    for i, (tid, ph, name, owner, days, deps, spec, acc, pct) in enumerate(T, start=2):
        ws.append([tid, ph, (name[:48]+"…") if len(name)>49 else name, MODEL[TIER[tid]], start[tid], end[tid], days, ",".join(deps), pct, None])
        ws.cell(i, 5).number_format = ws.cell(i, 6).number_format = "yyyy-mm-dd"
        ws.cell(i, 9).number_format = "0%"
        ws.cell(i, 10).value = f'=IF(I{i}>=1,"Done",IF(TODAY()>F{i},"Late",IF(I{i}>0,"In progress","Not started")))'
        ws.cell(i, 11).value = f'=G{i}*I{i}'
        pass
    n = len(T) + 1
    wc0, wc1 = L(len(cols) + 1), L(len(cols) + len(weeks))
    rng = f"{wc0}2:{wc1}{n}"
    # completed slice (dark) then planned (phase color); today marker column
    ws.conditional_formatting.add(rng, FormulaRule(formula=[f'AND({wc0}$1+6>=$E2,{wc0}$1<=$F2,{wc0}$1<=$E2+($F2-$E2)*$I2)'],
                                                   fill=PatternFill("solid", fgColor="1E8E3E"), stopIfTrue=True))
    for ph, (_, color) in PHASES.items():
        ws.conditional_formatting.add(rng, FormulaRule(formula=[f'AND(LEFT($B2,2)="{ph}",{wc0}$1+6>=$E2,{wc0}$1<=$F2)'],
                                                       fill=PatternFill("solid", fgColor=color), stopIfTrue=True))
    ws.conditional_formatting.add(rng, FormulaRule(formula=[f'AND(TODAY()>={wc0}$1,TODAY()<{wc0}$1+7)'],
                                                   fill=PatternFill("solid", fgColor="FFF4CE")))
    ws.conditional_formatting.add(f"J2:J{n}", FormulaRule(formula=['LEFT(J2,4)="Late"'], font=Font(color="D93025", bold=True)))
    ws.conditional_formatting.add(f"J2:J{n}", FormulaRule(formula=['J2="Done"'], font=Font(color="1E8E3E", bold=True)))
    for col, w in zip("ABCDEFGHIJK", [7, 22, 58, 14, 11, 11, 6, 16, 9, 12, 6]): ws.column_dimensions[col].width = w
    ws.freeze_panes = "D2"; ws.row_dimensions[1].height = 48
    # ---- Progress
    ps = wb.create_sheet("Progress")
    ps.append(["Phase", "Tasks", "Work days", "Done", "In progress", "Late", "% complete (effort-weighted)", "Start", "End", "Name"])
    for i, (ph, (pname, color)) in enumerate(PHASES.items(), start=2):
        key = ph
        ps.append([key, f'=COUNTIF(Gantt!$B:$B,A{i})', f'=SUMIF(Gantt!$B:$B,A{i},Gantt!$G:$G)',
                   f'=COUNTIFS(Gantt!$B:$B,A{i},Gantt!$J:$J,"Done")', f'=COUNTIFS(Gantt!$B:$B,A{i},Gantt!$J:$J,"In progress")',
                   f'=COUNTIFS(Gantt!$B:$B,A{i},Gantt!$J:$J,"Late*")',
                   f'=IF(C{i}>0,SUMIF(Gantt!$B:$B,A{i},Gantt!$K:$K)/C{i},0)',
                   min(start[t[0]] for t in T if t[1] == ph), max(end[t[0]] for t in T if t[1] == ph), pname])
        ps.cell(i, 1).fill = PatternFill("solid", fgColor=color)
    tot = len(PHASES) + 2
    ps.append(["Overall", f"=SUM(B2:B{tot-1})", f"=SUM(C2:C{tot-1})", f"=SUM(D2:D{tot-1})", f"=SUM(E2:E{tot-1})",
               f"=SUM(F2:F{tot-1})", f"=IF(C{tot}>0,SUM(Gantt!K2:K{n})/C{tot},0)",
               min(start.values()), max(end.values())])
    for r in range(2, tot + 1):
        ps.cell(r, 7).number_format = "0%"; ps.cell(r, 8).number_format = ps.cell(r, 9).number_format = "yyyy-mm-dd"
    for c in range(1, 10):
        ps.cell(1, c).font = hdr_font; ps.cell(1, c).fill = hdr_fill
        ps.cell(tot, c).font = Font(bold=True)
    for col, w in zip("ABCDEFGHI", [34, 7, 10, 7, 11, 7, 26, 12, 12]): ps.column_dimensions[col].width = w
    # ---- Milestones
    ms = wb.create_sheet("Milestones")
    ms.append(["Milestone", "Gate task", "Target date", "Status"])
    for i, (m, t) in enumerate(MILESTONES, start=2):
        ms.append([m, t, f"=INDEX(Gantt!$F:$F,MATCH(B{i},Gantt!$A:$A,0))", f"=INDEX(Gantt!$J:$J,MATCH(B{i},Gantt!$A:$A,0))"])
        ms.cell(i, 3).number_format = "yyyy-mm-dd"
    for c in range(1, 5): ms.cell(1, c).font = hdr_font; ms.cell(1, c).fill = hdr_fill
    for col, w in zip("ABCD", [44, 10, 13, 14]): ms.column_dimensions[col].width = w
    ps.cell(tot+2,1).value = "Edit only % Complete on the Gantt tab; everything else updates. Detail: docs/specs/10-tasks.md"
    wb.save(path)

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TASKS_JSON = os.path.join(ROOT, ".claude/harness/tasks.json")

def tasks_json():
    old = {t["id"]: t for t in json.load(open(TASKS_JSON))} if os.path.exists(TASKS_JSON) else {}
    out = []
    for tid, ph, name, owner, days, deps, spec, acc, pct in T:
        o = old.get(tid, {})
        out.append({"id": tid, "phase": ph, "name": name, "lane": lane.get(tid), "tier": TIER[tid], "model": MODEL[TIER[tid]],
                    "days": days, "deps": deps, "spec": spec, "acceptance": acc, "demo": demo(tid, ph, name),
                    "start": str(start[tid]), "end": str(end[tid]),
                    "status": o.get("status", "done" if pct >= 1 else "todo"), "pct": o.get("pct", pct)})
    return out

if __name__ == "__main__":
    tj = tasks_json()
    pct = {t["id"]: t["pct"] for t in tj}
    T[:] = [t[:8] + (pct[t[0]],) for t in T]
    json.dump(tj, open(TASKS_JSON, "w"), indent=1, ensure_ascii=False)
    open(os.path.join(ROOT, "docs/specs/10-tasks.md"), "w").write(md())
    xlsx(os.path.join(ROOT, "docs/space-compute-gantt-v3.xlsx"))
    print("end", max(end.values()), {m: str(end[t]) for m, t in MILESTONES})
