pub fn build_version(canister: &str) -> String {
    format!("{canister} {}", env!("CARGO_PKG_VERSION"))
}
