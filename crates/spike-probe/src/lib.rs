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

ic_cdk::export_candid!();
