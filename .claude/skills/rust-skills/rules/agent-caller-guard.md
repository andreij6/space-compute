# agent-caller-guard

> Enforce RAII `CallerGuard` on state-mutating async methods to prevent inter-canister reentrancy

## Why It Matters

Internet Computer canisters are single-threaded actors, but they process incoming calls cooperatively. When a method performs an inter-canister `.await`, execution is suspended, allowing other calls to execute before the await resumes.

If an agent writes an async method that checks a balance, awaits a transfer, and then updates state, a concurrent call from the same user can double-spend or bypass rate limits (Time-of-Check to Time-of-Use / TOCTOU vulnerability).

## Bad

```rust
// Vulnerable to reentrancy during .await
#[ic_cdk::update]
async fn buy_fuel_pack(pack_id: u64) -> Result<(), ApiError> {
    let caller = ic_cdk::caller();
    if state::has_pending_order(&caller) {
        return Err(ApiError::PendingOrderExists);
    }
    
    // SUSPENSION POINT: A concurrent call from caller can enter here!
    let transfer_res = ic_cdk::call(ledger_id, "icrc2_transfer_from", (args,)).await;
    
    state::credit_fuel(&caller, pack_id);
    Ok(())
}
```

## Good

```rust
use std::collections::HashSet;
use std::cell::RefCell;

thread_local! {
    static ACTIVE_CALLERS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
}

/// RAII Guard that releases the caller lock on Drop (even if panicked or trapped)
pub struct CallerGuard {
    caller: Principal,
}

impl CallerGuard {
    pub fn acquire(caller: Principal) -> Result<Self, ApiError> {
        ACTIVE_CALLERS.with(|set| {
            if !set.borrow_mut().insert(caller) {
                return Err(ApiError::ConcurrentCallDisallowed);
            }
            Ok(Self { caller })
        })
    }
}

impl Drop for CallerGuard {
    fn drop(&mut self) {
        ACTIVE_CALLERS.with(|set| set.borrow_mut().remove(&self.caller));
    }
}

#[ic_cdk::update]
async fn buy_fuel_pack(pack_id: u64) -> Result<(), ApiError> {
    let caller = ic_cdk::caller();
    let _guard = CallerGuard::acquire(caller)?; // Locked until function returns
    
    let transfer_res = ic_cdk::call(ledger_id, "icrc2_transfer_from", (args,)).await;
    state::credit_fuel(&caller, pack_id);
    Ok(())
}
```

## See Also

- [agent-saga-journal](./agent-saga-journal.md) - Saga journaling before await
- [agent-bounded-wait](./agent-bounded-wait.md) - Bounded wait on async calls
