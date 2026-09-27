# agent-schema-additive

> Evolve Candid and stable storage schemas strictly additively (`v: u8`, `Option<T>`, serde defaults)

## Why It Matters

In a distributed canister system, canisters are upgraded in-place while keeping their stable memory. If an agent changes the type of an existing field, renumbers fields, or removes an enum variant, reading prior state will fail decoding and trap the canister, permanently bricking it.

Following strict additive rules guarantees that any canister can be upgraded across versions without data migration traps.

## Additive Evolution Rules

1. **Schema Version Tag:** Every top-level stored struct must include a `pub v: u8` field.
2. **New Fields Must Be `Option<T>`:** Never add a non-optional field to an existing stored struct.
3. **Never Renumber or Reorder Struct Fields or Enum Variants:** Candid and serde binary decoders rely on consistent ordering and tags.
4. **Use `#[serde(default)]`:** Ensure old serialized data deserializes cleanly with defaults for newly added fields.

## Bad

```rust
// Version 1
#[derive(CandidType, Deserialize)]
pub struct UserRecord {
    pub principal: Principal,
    pub xp: u64,
}

// Version 2: BREAKING! Removed xp, added non-optional tier
#[derive(CandidType, Deserialize)]
pub struct UserRecord {
    pub principal: Principal,
    pub tier: u8, // TRAP on deserializing V1 state!
}
```

## Good

```rust
// Version 1
#[derive(CandidType, Deserialize)]
pub struct UserRecord {
    pub v: u8,
    pub principal: Principal,
    pub xp: u64,
}

// Version 2: Fully backward compatible
#[derive(CandidType, Deserialize)]
pub struct UserRecord {
    pub v: u8, // Bump to 2 on write
    pub principal: Principal,
    pub xp: u64,
    #[serde(default)]
    pub tier: Option<u8>, // None for V1 records
    #[serde(default)]
    pub badges: Vec<BadgeId>, // Empty vector for V1 records
}
```

## See Also

- [agent-contract-first](./agent-contract-first.md) - Contract freezing
- [agent-memory-id-range](./agent-memory-id-range.md) - MemoryId partitioning
