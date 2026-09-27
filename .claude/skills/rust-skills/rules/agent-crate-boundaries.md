# agent-crate-boundaries

> Enforce strict multi-crate workspace separation where each canister is an independent crate

## Why It Matters

Parallel agents should never touch the same files simultaneously. If multiple agents edit a monolithic crate, git conflicts on `Cargo.toml`, `mod.rs`, and internal utility files will disrupt the workflow.

Partitioning the project into distinct Cargo crates creates hard boundaries:
- `crates/sc-types`: Shared contracts, Candid types, error definitions
- `crates/platform`: Science registry, catalog, scoring, reviews
- `crates/payments`: Pass-through payments, fuel packs, treasury float
- `crates/treasury`: Cycles keeper, reserve monitoring
- `crates/aaa`: User Agent Amateur Astronomer canister wasm template
- `crates/integration-tests`: PocketIC end-to-end multi-canister tests

An agent assigned to a `platform` task touches **only** `crates/platform`, with zero probability of conflicting with an agent working on `crates/payments`.

## Bad

```
src/
├── lib.rs          # 27,000 lines (all canisters combined)
├── platform.rs
├── payments.rs
└── aaa.rs
# Every agent edits src/lib.rs, causing git merge conflicts.
```

## Good

```
Cargo.toml          # Workspace root
crates/
├── sc-types/       # Agent C (contract-first)
├── platform/       # Agent A (Lane 1)
├── payments/       # Agent B (Lane 2)
├── treasury/       # Agent D (Lane 3)
└── integration-tests/
```

## Workspace Configuration Pattern

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/sc-types",
    "crates/platform",
    "crates/payments",
    "crates/treasury",
    "crates/aaa",
    "crates/integration-tests",
]

[workspace.dependencies]
candid = "0.10"
ic-cdk = "0.19"
ic-stable-structures = "0.6"
serde = { version = "1.0", features = ["derive"] }
sc-types = { path = "crates/sc-types" }
```

## See Also

- [proj-workspace-large](./proj-workspace-large.md) - Workspaces for large projects
- [agent-micro-modules](./agent-micro-modules.md) - Micro-module rule
