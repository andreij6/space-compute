# agent-task-isolated-tests

> Create dedicated, self-contained test files per task (`tests/test_T*.rs`) with independent fixtures

## Why It Matters

When multiple agents run test suites in parallel, shared test files or shared mutable test state cause race conditions and flakey test failures.

Each task specification in `tasks.json` has an associated acceptance test. Creating a dedicated test file named after the task ID (e.g. `tests/test_T2_4_scoring.rs`) allows:
1. An agent to run only its own test suite: `cargo test --test test_T2_4_scoring` without interfering with other lanes.
2. Acceptance demos to execute cleanly via `just demo T2.4`.
3. Independent PocketIC instances so state never leaks between tests.

## Bad

```
tests/
└── all_tests.rs    # Monolithic 5,000-line test file shared across all tasks
```

## Good

```
tests/
├── common/
│   ├── mod.rs      # Shared PocketIC setup helpers & wasm loaders
│   └── fixtures.rs # Deterministic test constants (never mutate!)
├── test_T1_2_types.rs
├── test_T2_1_platform_skeleton.rs
├── test_T2_4_scoring.rs
└── test_T3_2_forwarding.rs
```

## Self-Contained Test Template

```rust
// tests/test_T2_4_scoring.rs
mod common;

#[test]
fn test_t2_4_scoring_retirement_at_k5() {
    let pic = common::setup_pocket_ic();
    let platform = common::deploy_platform(&pic);
    
    // Test runs isolated in its own replica instance
    println!("✓ T2.4: Seeded 5 classifications for subject 42...");
    // Assertions...
}
```

## Gate Requirement

- Disallow test skips from masking failures (`L-007`).
- Always assert test count $> 0$.

## See Also

- [test-arrange-act-assert](./test-arrange-act-assert.md) - Standard test structure
- [agent-hexagonal-pure](./agent-hexagonal-pure.md) - Pure domain logic tests
