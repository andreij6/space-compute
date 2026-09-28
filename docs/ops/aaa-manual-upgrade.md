# AAA manual upgrade (self-managed owners)

Spec: `docs/specs/08-security.md` §7, `docs/specs/03-aaa-canister.md` §7.

An AAA owner may remove `platform` as a controller of their canister at any
time (03 §7). The AAA keeps working — it never depended on `platform` being
a controller for normal operation — but `platform` can no longer push
upgrades for it. From then on, upgrading that AAA's wasm is entirely the
owner's job, and this is how to do it safely.

## 1. Build the wasm reproducibly

```bash
just aaa-reproducible
```

This runs `scripts/build-aaa-reproducible.sh`, which rebuilds `crates/aaa`
with the same flags `icp.yaml`'s `@dfinity/rust` recipe uses — release
profile, `wasm32-unknown-unknown`, `ic-wasm shrink`, then `gzip -n` (no
embedded name or mtime) — inside the pinned toolchain (`rust-toolchain.toml`,
`ic-wasm 0.11.1`). It builds in a container (`Dockerfile.aaa-reproducible`,
pinned by digest) when Docker or Podman is reachable, and otherwise falls
back to a hermetic local build: an isolated `CARGO_HOME`/target directory
per run and `--remap-path-prefix` on the source tree, cargo home, and rustc
sysroot, so the absolute path you happen to be building from never leaks
into the wasm bytes.

It prints:

```
module_sha256=<sha256 of the shrunk .wasm>
gz_sha256=<sha256 of the gzipped .wasm.gz>
```

`module_sha256` is what matters: it is what `crates/platform/src/registry.rs`
stores as `WasmMeta.module_sha256`, and — because the IC decompresses a
gzip-installed module before hashing it — it is exactly what
`icp canister status` reports as `module_hash` once the wasm is installed,
whether it was installed from the raw `.wasm` or the `.wasm.gz`.

### Prove it's reproducible, not a fluke

```bash
just aaa-reproducible-check
```

`scripts/aaa-reproducible-check.sh` copies the current source into two
unrelated absolute directories (e.g. `/tmp/sc-aaa-repro-.../machine-a` and a
deeply nested `.../machine-b/deeply/nested/checkout`), builds each with a
different `$USER`/`$LANG` and an isolated `CARGO_HOME`/target, and fails
loudly if `module_sha256` or `gz_sha256` differ. This is the "2 machines"
acceptance check — an owner (or a second maintainer) runs this same script
on their own machine and expects it to print the identical hashes below.
It is not part of `just verify` (a full release build takes longer than the
fast gate); run it once per release and whenever you doubt a hash.

## 2. Verify the hash against what the platform approved

Before installing anything, confirm `module_sha256` from step 1 matches an
**approved** `WasmMeta.module_sha256` for the AAA wasm version you intend to
run:

- **Published table** (below): the owner (or a maintainer with release
  access) records `module_sha256`/`gz_sha256` here for every version they
  approve, per 08 §7 ("publish reproducible-build instructions... and the
  sha256 for each approved version"). This is the source most self-managed
  owners use, since they are not necessarily platform admins.
- **`admin_list_wasms`** (admin-gated query on `platform`, returns
  `vec record { nat32; WasmMeta }` per `platform.did`): if you hold a
  platform admin identity, cross-check directly —
  `icp canister call platform admin_list_wasms --identity <admin-identity> -e <env>`
  and compare the `module_sha256` field for your target version. Ordinary
  operator/owner identities cannot call this (S11); it is a cross-check for
  admins, not the primary path.

Never install a wasm whose `module_sha256` isn't in the published table (or
isn't `approved: true` via `admin_list_wasms`). `platform` re-verifies the
AAA's on-chain module hash at the next call regardless (S8) and suspends the
AAA on a mismatch, so an unapproved or tampered wasm gets caught even if you
skip this step — but catching it before you spend an upgrade call is cheaper.

### Published AAA wasm versions

| Version | module_sha256 | gz_sha256 | Released |
|---|---|---|---|
| _(none released yet — fill in at the first real AAA release)_ | | | |

## 3. Install the upgrade

Never `dfx`. Always pass `--identity` and `-e`/`--environment` explicitly.

```bash
# Mainnet command shown for reference only — never run this automatically.
icp canister install aaa --mode upgrade \
  --wasm target/aaa-reproducible/local/aaa.wasm.gz \
  --identity <owner-identity> \
  -e production
```

`--mode upgrade` runs `pre_upgrade`/`post_upgrade` and keeps stable memory
(03 §1, §3) — operators, records, and credits all survive.

## 4. Confirm the upgrade took

```bash
# Mainnet command shown for reference only — never run this automatically.
icp canister status aaa --identity <owner-identity> -e production
```

Compare the printed `module_hash` to the `module_sha256` you verified in
step 2. They must match exactly. `platform`'s own 24 h heartbeat check (03
§6, 08 S8) performs the same comparison independently and suspends the AAA
if it ever disagrees — this step just confirms it before that timer runs.

## Local practice run

Everything above works against the local network first:

```bash
just deploy-local
just aaa-reproducible
icp canister install aaa --mode upgrade --wasm target/aaa-reproducible/local/aaa.wasm.gz --identity sc-user -e local
icp canister status aaa --identity sc-user -e local
```
