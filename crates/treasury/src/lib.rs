pub mod api;
pub mod config;
pub mod keeper;
mod memory;
pub mod report;
pub mod state;
pub mod timers;

use candid::Principal;
use config::Config;
use report::{Health, Status};
use sc_types::ApiError;
use state::{HistoryItem, Proposal};

#[ic_cdk::init]
fn init() {
    let installer = ic_cdk::api::msg_caller();
    if installer == Principal::anonymous() {
        ic_cdk::trap("treasury must be installed by a non-anonymous principal");
    }
    config::set(Config {
        admins: vec![installer],
        ..Config::default()
    })
    .expect("init config");
    timers::start();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    let upgrader = ic_cdk::api::msg_caller();
    let current = config::get();
    if upgrader != Principal::anonymous() && current.admins.is_empty() {
        config::set(Config {
            admins: vec![upgrader],
            ..current
        })
        .expect("bootstrap admin");
    }
    timers::start();
}

#[ic_cdk::query]
fn version() -> String {
    sc_types::build_version("treasury")
}

ic_cdk::export_candid!();
