#!/usr/bin/env bash
# PreToolUse(Bash) guard. Exit 2 = block (stderr is shown to Claude).
# Promoted lessons: L-008 (mainnet), dfx ban (CLAUDE.md).
cmd=$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("command",""))')
block() { echo "harness guard: $1" >&2; exit 2; }
[[ "$cmd" =~ (^|[[:space:];&|])dfx([[:space:]]|$) ]] && block "use the icp CLI, never dfx"
if [[ "$cmd" =~ (-e|--environment)[[:space:]=]+(ic|production|staging) || "$cmd" =~ --network[[:space:]=]+ic ]]; then
  [[ "${SC_ALLOW_MAINNET:-}" == "1" ]] || block "mainnet/staging command blocked. Only when the owner asked this session; rerun with SC_ALLOW_MAINNET=1"
fi
[[ "$cmd" =~ --mode[[:space:]=]+reinstall && ! "$cmd" =~ -e[[:space:]]+local ]] && block "reinstall wipes state; only allowed with -e local"
if [[ "$cmd" =~ (^|[[:space:];&|])icp[[:space:]]+(canister|deploy|token|cycles)([[:space:]]|$) && ! "$cmd" =~ --identity ]]; then
  block "pass --identity explicitly (sc-deployer/sc-admin/sc-user/anonymous); the default identity here is a password-protected mainnet key and hangs waiting for a password (L-008)"
fi
exit 0
