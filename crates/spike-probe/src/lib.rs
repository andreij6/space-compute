use candid::{CandidType, Principal};
use ic_cdk::call::Call;
use serde::Deserialize;

#[derive(CandidType)]
struct CanisterInfoArgs {
    canister_id: Principal,
    num_requested_changes: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct CanisterInfoResult {
    total_num_changes: u64,
    module_hash: Option<Vec<u8>>,
    controllers: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct InfoProbe {
    pub total_num_changes: u64,
    pub module_hash_len: u32,
    pub controllers: u32,
    pub cycles_spent: u64,
    pub instructions: u64,
}

#[ic_cdk::update]
async fn probe_canister_info(target: Principal) -> Result<InfoProbe, String> {
    let before = ic_cdk::api::canister_cycle_balance();
    let reply = Call::bounded_wait(Principal::management_canister(), "canister_info")
        .with_arg(CanisterInfoArgs {
            canister_id: target,
            num_requested_changes: None,
        })
        .await
        .map_err(|e| format!("{e:?}"))?;
    let info: CanisterInfoResult = reply.candid().map_err(|e| format!("{e:?}"))?;
    Ok(InfoProbe {
        total_num_changes: info.total_num_changes,
        module_hash_len: info.module_hash.map(|h| h.len() as u32).unwrap_or(0),
        controllers: info.controllers.len() as u32,
        cycles_spent: (before - ic_cdk::api::canister_cycle_balance()) as u64,
        instructions: ic_cdk::api::performance_counter(1),
    })
}

#[derive(CandidType, Deserialize)]
pub struct SubnetCosts {
    pub create_canister: u128,
    pub call_empty: u128,
    pub call_per_kib: u128,
}

#[ic_cdk::update]
fn subnet_costs() -> SubnetCosts {
    let call_empty = ic_cdk::api::cost_call(0, 0);
    SubnetCosts {
        create_canister: ic_cdk::api::cost_create_canister(),
        call_empty,
        call_per_kib: ic_cdk::api::cost_call(0, 1_024) - call_empty,
    }
}

#[ic_cdk::update]
fn burn(rounds: u64) -> u64 {
    let mut acc = 0u64;
    for i in 0..rounds {
        acc = std::hint::black_box(acc.wrapping_mul(31).wrapping_add(i));
    }
    std::hint::black_box(acc);
    ic_cdk::api::performance_counter(0)
}

ic_cdk::export_candid!();
