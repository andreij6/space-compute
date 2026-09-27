# agent-micro-modules

> Keep source files under 300–500 lines with single responsibility; avoid monolithic files

## Why It Matters

Monolithic files create two severe bottlenecks for AI agents:
1. **Context Window Exhaustion:** An agent must spend 15k–30k tokens just reading the file before making a 10-line edit, draining token budgets and degrading reasoning.
2. **File-Level Lock Contention:** If two tasks in the same crate touch different endpoints in the same file, git conflicts arise.

In `proof-of-burn`, a 27,000-line `lib.rs` required a custom section-map navigation skill to make progress. In Space Compute, no file may exceed ~500 lines.

## File Breakdown per Canister Crate

Inside any canister crate (`crates/platform/src/`):

```
crates/platform/src/
├── lib.rs          # Module declarations, init, post_upgrade (~80 lines)
├── endpoints.rs    # Candid query/update function signatures (~200 lines)
├── state.rs        # MemoryManager & StableBTreeMap wrappers (~150 lines)
├── memory.rs       # MemoryId allocation table (~60 lines)
├── guards.rs       # Caller validation, admin checks, rate limiters (~120 lines)
├── scoring.rs      # Pure domain logic for classification tallies (~250 lines)
├── catalog.rs      # Pure domain logic for subject leases (~200 lines)
└── reviews.rs      # Pure domain logic for blind review assignment (~250 lines)
```

## Bad

```rust
// lib.rs (3,000 lines)
// Contains state definition, endpoints, scoring algorithms,
// candid export, and internal helpers all in one place.
```

## Good

```rust
// lib.rs (thin orchestration)
mod endpoints;
mod guards;
mod memory;
mod state;
mod scoring;

pub use endpoints::*;

#[ic_cdk::init]
fn init() {
    state::init();
}
```

## Benefits for Parallel Agents

- Agent 1 works on scoring algorithms (`scoring.rs`).
- Agent 2 adds review logic (`reviews.rs`).
- Both pull requests or commits merge cleanly without touching each other's code.

## See Also

- [proj-mod-by-feature](./proj-mod-by-feature.md) - Organize modules by feature
- [agent-hexagonal-pure](./agent-hexagonal-pure.md) - Pure domain logic separation
