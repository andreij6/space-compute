# agent-hexagonal-pure

> Decouple pure business logic from IC runtime calls (`ic_cdk`), enabling instant parallel unit testing

## Why It Matters

Code that directly invokes `ic_cdk::caller()`, `ic_cdk::api::time()`, or `ic_cdk::call` cannot be tested with native `cargo test`. It requires PocketIC, a running replica, or complex test mocking harnesses.

By separating domain algorithms (scoring, consensus, XP calculations, rate-limiting windows, tier progression) into **pure Rust functions** that take inputs and return outputs, agents can write and run unit tests in milliseconds (`cargo test -p platform`) without waiting for WASM compilation or canister deployment.

## Bad

```rust
// Tightly coupled to ic_cdk runtime
pub fn submit_classification(subject_id: u64, label: Classification) -> Result<Reward, ApiError> {
    let caller = ic_cdk::caller(); // Impossible to unit test natively!
    let now = ic_cdk::api::time(); // Impossible to unit test natively!
    
    if is_rate_limited(caller, now) {
        return Err(ApiError::RateLimited);
    }
    // ...
}
```

## Good

```rust
// Pure domain logic in scoring.rs (100% testable natively with cargo test)
pub struct ClassificationContext {
    pub caller: Principal,
    pub now_ns: u64,
    pub subject_id: u64,
    pub current_gold: Option<Classification>,
}

pub fn evaluate_classification(
    ctx: &ClassificationContext,
    history: &[SubmissionRecord],
) -> Result<EvaluationOutcome, ApiError> {
    // Pure logic: no ic_cdk calls here
    // Deterministic, easily fuzzable, instantly testable
}

// Thin endpoint adapter in endpoints.rs
#[ic_cdk::update]
pub fn submit_classification(subject_id: u64, label: Classification) -> Result<Reward, ApiError> {
    let caller = guards::require_authenticated_caller()?;
    let now_ns = ic_cdk::api::time();
    let ctx = ClassificationContext { caller, now_ns, subject_id, ... };
    
    let outcome = scoring::evaluate_classification(&ctx, &history)?;
    state::record_outcome(outcome);
    Ok(outcome.reward)
}
```

## Benefits for Parallel Agents

- Any agent implementing a feature writes comprehensive `cargo test` unit tests in seconds.
- Parallel agents don't block each other waiting for PocketIC wasms to build.

## See Also

- [test-arrange-act-assert](./test-arrange-act-assert.md) - AAA test pattern
- [agent-task-isolated-tests](./agent-task-isolated-tests.md) - Task-isolated test suites
