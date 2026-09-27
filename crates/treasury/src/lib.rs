#[ic_cdk::init]
fn init() {}

#[ic_cdk::post_upgrade]
fn post_upgrade() {}

#[ic_cdk::query]
fn version() -> String {
    sc_types::build_version("treasury")
}

ic_cdk::export_candid!();
