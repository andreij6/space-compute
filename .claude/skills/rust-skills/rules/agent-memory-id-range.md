# agent-memory-id-range

> Centralize and partition `MemoryId` ranges so concurrent storage additions never collide

## Why It Matters

In `ic-stable-structures`, each `StableBTreeMap` or `StableCell` is bound to a `MemoryId(u8)`. If Agent A creates a new table with `MemoryId::new(5)` while Agent B working on a different feature also picks `MemoryId::new(5)`, the two maps share the same virtual memory space—causing silent, catastrophic data corruption upon upgrade.

Centralizing all `MemoryId` assignments into an explicit, partitioned `memory.rs` file prevents conflicts.

## Bad

```rust
// In module_a.rs:
static MAP_A: RefCell<StableBTreeMap<u64, User, Memory>> = 
    RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))));

// In module_b.rs:
static MAP_B: RefCell<StableBTreeMap<u64, Discovery, Memory>> = 
    RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))); // COLLISION!
```

## Good

```rust
// memory.rs (Centralized registry with reserved sub-ranges)
use ic_stable_structures::memory_manager::MemoryId;

// Range 0..9: Platform Core & Config
pub const MEMORY_ID_CONFIG: MemoryId = MemoryId::new(0);
pub const MEMORY_ID_AAA_REGISTRY: MemoryId = MemoryId::new(1);
pub const MEMORY_ID_USERS: MemoryId = MemoryId::new(2);

// Range 10..19: Catalog & Leases
pub const MEMORY_ID_SUBJECTS: MemoryId = MemoryId::new(10);
pub const MEMORY_ID_LEASES: MemoryId = MemoryId::new(11);
pub const MEMORY_ID_SEEN_SET: MemoryId = MemoryId::new(12);

// Range 20..29: Reviews & Scoring
pub const MEMORY_ID_CLASSIFICATIONS: MemoryId = MemoryId::new(20);
pub const MEMORY_ID_REVIEWS: MemoryId = MemoryId::new(21);
pub const MEMORY_ID_DISCOVERIES: MemoryId = MemoryId::new(22);

// Range 30..39: Event Log & Certifications
pub const MEMORY_ID_EVENT_LOG: MemoryId = MemoryId::new(30);
pub const MEMORY_ID_EVENT_INDEX: MemoryId = MemoryId::new(31);
pub const MEMORY_ID_CITATIONS: MemoryId = MemoryId::new(32);
```

## Storage Invariants

1. **Never renumber an existing `MemoryId`:** Doing so corrupts stable storage across canister upgrades.
2. **Assign by range:** When an agent works on a feature area, it claims IDs from that area's allocated range.

## See Also

- [agent-schema-additive](./agent-schema-additive.md) - Additive schema changes
