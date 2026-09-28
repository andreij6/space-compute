pub mod api;
pub mod audit;
pub mod cmc;
pub mod config;
pub mod deposit;
pub mod guard;
pub mod invites;
pub mod journal;
pub mod ledger;
pub mod mandate;
mod memory;
pub mod owners;
pub mod platform_client;
pub mod quote;
pub mod rate;
mod timers;

use api::{
    JournalDemoArg, MintInvitesArgs, PaymentsOverview, SetMandateArgs, SpawnArgs, TopUpArgs,
    TreasuryWithdrawArgs,
};
use audit::AuditEntry;
use candid::Principal;
use config::{Features, Params, PauseFlags};
use deposit::Purpose;
use journal::{Account, Op, OpFilter, Page};
use mandate::MandateView;
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
    let upgrader = ic_cdk::api::msg_caller();
    if upgrader != Principal::anonymous() {
        config::update(|c| {
            if c.admins.is_empty() {
                c.admins = vec![upgrader];
            }
            Ok(())
        })
        .expect("bootstrap admin");
    }
    journal::backfill_indexes();
    timers::start();
}

#[ic_cdk::query]
fn version() -> String {
    sc_types::build_version("payments")
}

ic_cdk::export_candid!();
