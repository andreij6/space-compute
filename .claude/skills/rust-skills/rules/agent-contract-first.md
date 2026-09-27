# agent-contract-first

> Define and freeze shared types and Candid interfaces in a dedicated shared crate before implementing canisters

## Why It Matters

When multiple autonomous agents work in parallel, inter-agent dependencies are the #1 source of merge conflicts and compilation breaks. If Agent A implements a caller while Agent B is still modifying the callee's API, both will stumble into broken builds and token-burning retry loops.

By freezing the contract (Candid interfaces, shared structs, event enums, and `ApiError` variants) in a shared types crate (`crates/sc-types`) first, multiple agents can write producers, consumers, mocks, and tests simultaneously without coordination.

## Bad

```rust
// Agent A puts types directly inside platform canister crate
// crates/platform/src/lib.rs
pub struct SubjectDossier {
    pub id: u64,
    pub classification: String,
}

// Agent B (working on AAA canister) needs SubjectDossier, but cannot depend
// on platform canister directly (circular dependency or heavy wasm bloat).
// Agent B duplicates the struct or waits for Agent A.
```

## Good

```rust
// crates/sc-types/src/catalog.rs (Shared interface crate)
use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub struct SubjectDossier {
    pub id: u64,
    pub title: String,
    pub ra: f64,
    pub dec: f64,
    pub v: u8, // Schema version
}

// crates/platform/src/endpoints.rs (Agent A)
use sc_types::catalog::SubjectDossier;

// crates/aaa/src/client.rs (Agent B)
use sc_types::catalog::SubjectDossier;
```

## Agent Parallelism Rule

1. Phase 1 must complete the shared types crate before starting parallel canister lanes.
2. Changes to shared contracts are **additive-only** (`Option<T>`, new enum variants with default handling).
3. Any contract modification requires an explicit schema version bump (`v: u8`).

## See Also

- [agent-schema-additive](./agent-schema-additive.md) - Additive Candid schema evolution
- [agent-crate-boundaries](./agent-crate-boundaries.md) - Multi-crate workspace boundaries
