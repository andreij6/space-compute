pub mod api;
pub mod audit;
pub mod config;
pub mod deposit;
pub mod guard;
pub mod journal;
mod memory;
pub mod quote;
pub mod rate;
mod timers;

use api::{JournalDemoArg, Overview};
use audit::AuditEntry;
use candid::Principal;
use config::{Features, Params, PauseFlags};
use deposit::Purpose;
use journal::{Account, Op};
use quote::Quote;
use rate::RateCache;
use sc_types::ApiError;

#[ic_cdk::init]
fn init() {
    let installer = ic_cdk::api::msg_caller();
    if installer == Principal::anonymous() {
        ic_cdk::trap("payments must be installed by a non-anonymous principal");
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
    sc_types::build_version("payments")
}

ic_cdk::export_candid!();
