pub mod api;
pub mod burn;
pub mod config;
pub mod credits;
pub mod doc;
pub mod forwarding;
mod memory;
pub mod operators;
pub mod params;
pub mod record;
pub mod repository;
pub mod roles;
mod timers;

use candid::Principal;
use config::AaaInit;
use operators::Operator;
use record::{ListRecordsFilter, PageCreditCopy, PageRecord, PublicStatus, Record, Status};
use roles::Role;
use sc_types::ApiError;

#[ic_cdk::init]
fn init(init_arg: AaaInit) {
    let cfg = config::Config::from_init(init_arg)
        .unwrap_or_else(|e| ic_cdk::trap(format!("invalid aaa init: {e}")));
    config::set(cfg);
    timers::start();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    timers::start();
}

#[ic_cdk::inspect_message]
fn inspect_message() {
    let caller = ic_cdk::api::msg_caller();
    let cfg = config::get();
    let now = ic_cdk::api::time();
    let operator_active = operators::is_active(&caller, now);
    if roles::can_ingress(caller, cfg.owner, operator_active) {
        ic_cdk::api::accept_message();
    }
}

#[ic_cdk::query]
fn version() -> String {
    sc_types::build_version("aaa")
}

ic_cdk::export_candid!();
