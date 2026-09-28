#!/usr/bin/env bash
# Sourced by scripts that touch a canister environment: refuses the machine-default identity,
# requires SC_ALLOW_MAINNET=1 plus a typed confirmation for anything but local, and runs icp
# calls that must succeed (non-zero exit or an Err variant aborts the caller).

guard_die() { echo -e "\033[0;31m✘\033[0m $1" >&2; exit 1; }

mainnet_guard() {
  local env="$1" identity="$2"
  [ -n "$identity" ] || guard_die "set the identity explicitly (sc-deployer for local); never rely on the machine default (prod-deployer)"
  [ "$identity" != "prod-deployer" ] || guard_die "refusing identity prod-deployer: it is the password-protected machine default, use a named release identity"
  [ "$env" = "local" ] && return 0
  [ "${SC_ALLOW_MAINNET:-}" = "1" ] || guard_die "refusing $env: rerun with SC_ALLOW_MAINNET=1 only when the owner asked this session"
  echo "About to run $(basename "$0") against $env as $identity. Type '$env' to confirm:"
  local confirm
  read -r confirm
  [ "$confirm" = "$env" ] || guard_die "confirmation mismatch, aborting"
}

icp_must() {
  local out
  out=$(icp "$@" 2>&1) || guard_die "icp $1 $2 $3 $4 failed: $out"
  if grep -Eq '\bErr\b' <<<"$out"; then
    guard_die "icp $1 $2 $3 $4 returned Err: $out"
  fi
  printf '%s\n' "$out"
}
