pub mod api;
pub mod audit;
pub mod catalog;
pub mod config;
pub mod events;
mod memory;
pub mod progression;
pub mod registry;
mod rng;
pub mod scoring;
mod timers;

use api::Overview;
use audit::AuditEntry;
use candid::Principal;
use catalog::{AdminListSubjectsFilter, Lease, Subject, SubjectInput};
use config::{Params, PauseFlags};
use registry::{
    AaaRecord, AdminListAaasFilter, CheckNameResult, Heartbeat, OperatorSetInput, RegisterArgs,
    UpdateAaaProfileArgs, WasmMeta,
};
use sc_types::ApiError;

#[ic_cdk::init]
fn init() {
    let installer = ic_cdk::api::msg_caller();
    if installer == Principal::anonymous() {
        ic_cdk::trap("platform must be installed by a non-anonymous principal");
    }
    config::update(|c| {
        c.admins = vec![installer];
        Ok(())
    })
    .expect("init config");
    timers::start();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    timers::start();
}

#[ic_cdk::query]
fn version() -> String {
    sc_types::build_version("platform")
}

ic_cdk::export_candid!();
