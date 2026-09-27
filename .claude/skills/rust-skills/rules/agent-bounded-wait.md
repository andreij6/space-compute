# agent-bounded-wait

> Use bounded-wait and explicit timeouts on all inter-canister calls to prevent deadlocks

## Why It Matters

Unbounded inter-canister calls (`ic_cdk::call`) wait indefinitely if the target canister is stopped, upgrading, or unresponsive. In a multi-canister architecture, an unbounded wait can hang a canister's execution queue and exhaust its instruction limits.

Using bounded wait (`Call::bounded_wait` in `ic-cdk`) ensures every remote call finishes or fails within an explicit timeout window.

## Bad

```rust
// Unbounded wait: if target is frozen or upgrading, this hangs indefinitely
let res = ic_cdk::call(target_canister, "process_task", (task,)).await;
```

## Good

```rust
use ic_cdk::api::call::Call;

// Bounded wait with explicit timeout
let res = Call::bounded_wait(target_canister, "process_task")
    .with_args((task,))
    .with_timeout_secs(15) // Maximum 15 seconds before timeout error
    .await;

match res {
    Ok(reply) => Ok(reply),
    Err(err) if err.is_timeout() => {
        ic_cdk::println!("Call to target timed out after 15s");
        Err(ApiError::RemoteCallTimeout)
    }
    Err(err) => Err(ApiError::RemoteCallFailed(err.to_string())),
}
```

## Concurrency Invariant

Always pair bounded wait with the [CallerGuard](./agent-caller-guard.md) and [Saga Journal](./agent-saga-journal.md) patterns to handle timeouts gracefully without leaking state.

## See Also

- [agent-caller-guard](./agent-caller-guard.md) - Reentrancy prevention
- [agent-saga-journal](./agent-saga-journal.md) - Saga journaling
