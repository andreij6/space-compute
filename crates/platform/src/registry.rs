use std::cell::RefCell;
use std::io::Read;

use candid::{CandidType, Principal};
use flate2::read::GzDecoder;
use ic_stable_structures::StableBTreeMap;
use sc_types::ApiError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::memory::{self, Memory};

pub const MAX_WASM_BYTES: usize = 1_887_436;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WasmMeta {
    pub sha256: Vec<u8>,
    pub size: u64,
    pub approved: bool,
    pub released_at: u64,
}

crate::candid_storable!(WasmMeta);

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AaaStatus {
    Installing,
    Active,
    Suspended,
    SelfManaged,
    Deleted,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct AaaRecord {
    pub v: u8,
    pub owner: Principal,
    pub name: String,
    pub avatar_seed: u64,
    pub wasm_version: u32,
    pub status: AaaStatus,
    pub created_at: u64,
    pub last_seen_at: u64,
    pub last_cycles: u128,
    pub platform_is_controller: bool,
    pub verified_at: u64,
    pub install_attempts: u8,
}

crate::candid_storable!(AaaRecord);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OperatorSet {
    pub owner: Principal,
    pub operators: Vec<(Principal, Option<u64>)>,
    pub synced_at: u64,
}

crate::candid_storable!(OperatorSet);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    pub module_hash: Vec<u8>,
    pub total_num_changes: u64,
    pub checked_at: u64,
}

crate::candid_storable!(Provenance);

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RegisterArgs {
    pub canister_id: Principal,
    pub owner: Principal,
    pub name: String,
    pub avatar_seed: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Heartbeat {
    pub cycles: u128,
    pub wasm_version: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct OperatorSetInput {
    pub operators: Vec<(Principal, Option<u64>)>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateAaaProfileArgs {
    pub name: Option<String>,
    pub avatar_seed: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckNameResult {
    Ok,
    Taken,
    Invalid,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct AdminListAaasFilter {
    pub status: Option<AaaStatus>,
    pub name_prefix: Option<String>,
    pub owner: Option<Principal>,
}

thread_local! {
    static WASM_STORE: RefCell<StableBTreeMap<u32, Vec<u8>, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::WASM_STORE)));
    static WASM_META: RefCell<StableBTreeMap<u32, WasmMeta, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::WASM_META)));
    static AAA_REGISTRY: RefCell<StableBTreeMap<Principal, AaaRecord, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_REGISTRY)));
    static AAA_OWNERS: RefCell<StableBTreeMap<Principal, Principal, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_OWNERS)));
    static AAA_NAMES: RefCell<StableBTreeMap<String, Principal, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_NAMES)));
    static AAA_OPERATORS: RefCell<StableBTreeMap<Principal, OperatorSet, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_OPERATORS)));
    static AAA_PROVENANCE: RefCell<StableBTreeMap<Principal, Provenance, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_PROVENANCE)));
}

pub fn upload_wasm(version: u32, blob: Vec<u8>, sha256: Vec<u8>) -> Result<(), ApiError> {
    if version == 0 {
        return Err(ApiError::invalid("wasm version must be > 0"));
    }
    if blob.is_empty() {
        return Err(ApiError::invalid("wasm blob cannot be empty"));
    }
    if blob.len() > MAX_WASM_BYTES {
        return Err(ApiError::invalid("wasm exceeds 1.8 MiB cap"));
    }
    let actual_hash = Sha256::digest(&blob).to_vec();
    if actual_hash != sha256 {
        return Err(ApiError::invalid("sha256 mismatch"));
    }
    WASM_STORE.with_borrow_mut(|m| m.insert(version, blob.clone()));
    WASM_META.with_borrow_mut(|m| {
        m.insert(
            version,
            WasmMeta {
                sha256,
                size: blob.len() as u64,
                approved: false,
                released_at: 0,
            },
        )
    });
    Ok(())
}

pub fn approve_wasm(version: u32, now: u64) -> Result<(), ApiError> {
    WASM_META.with_borrow_mut(|m| match m.get(&version) {
        Some(mut meta) => {
            meta.approved = true;
            meta.released_at = now;
            m.insert(version, meta);
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })
}

pub fn list_wasms() -> Vec<(u32, WasmMeta)> {
    WASM_META.with_borrow(|m| m.iter().map(|e| (*e.key(), e.value())).collect())
}

pub fn get_wasm(version: u32) -> Option<Vec<u8>> {
    WASM_STORE.with_borrow(|m| m.get(&version))
}

pub fn get_wasm_meta(version: u32) -> Option<WasmMeta> {
    WASM_META.with_borrow(|m| m.get(&version))
}

pub fn latest_approved_wasm() -> Option<(u32, WasmMeta, Vec<u8>)> {
    let metas = list_wasms();
    let (v, meta) = metas
        .into_iter()
        .filter(|(_, m)| m.approved)
        .max_by_key(|(v, _)| *v)?;
    let blob = get_wasm(v)?;
    Some((v, meta, blob))
}

pub fn is_approved_module_hash(target_hash: &[u8]) -> bool {
    let metas = list_wasms();
    for (v, meta) in metas.into_iter().filter(|(_, m)| m.approved) {
        if meta.sha256 == target_hash {
            return true;
        }
        if let Some(blob) = get_wasm(v) {
            if blob.starts_with(&[0x1f, 0x8b]) {
                let mut decoder = GzDecoder::new(&blob[..]);
                let mut uncompressed = Vec::new();
                if decoder.read_to_end(&mut uncompressed).is_ok()
                    && Sha256::digest(&uncompressed).as_slice() == target_hash
                {
                    return true;
                }
            }
        }
    }
    false
}

pub fn check_name(name: &str) -> CheckNameResult {
    match sc_types::limits::aaa_name(name) {
        Err(_) => CheckNameResult::Invalid,
        Ok(valid) => {
            let key = sc_types::limits::name_key(&valid);
            let exists = AAA_NAMES.with_borrow(|m| m.contains_key(&key));
            if exists {
                CheckNameResult::Taken
            } else {
                CheckNameResult::Ok
            }
        }
    }
}

pub fn resolve_name_for_spawn(raw_name: &str, canister_id: &Principal) -> Result<String, ApiError> {
    let base_name = sc_types::limits::aaa_name(raw_name)?;
    let base_key = sc_types::limits::name_key(&base_name);
    let owner_of_base = AAA_NAMES.with_borrow(|m| m.get(&base_key));
    if owner_of_base.is_none() || owner_of_base.as_ref() == Some(canister_id) {
        return Ok(base_name);
    }
    for suffix in 2..=1000 {
        let candidate = format!("{base_name}-{suffix}");
        if sc_types::limits::aaa_name(&candidate).is_err() {
            continue;
        }
        let key = sc_types::limits::name_key(&candidate);
        let owner = AAA_NAMES.with_borrow(|m| m.get(&key));
        if owner.is_none() || owner.as_ref() == Some(canister_id) {
            return Ok(candidate);
        }
    }
    Err(ApiError::Conflict("could not allocate unique name".into()))
}

pub fn pre_register_aaa(
    args: &RegisterArgs,
    now: u64,
) -> Result<(String, u32, Vec<u8>, bool), ApiError> {
    if args.owner == Principal::anonymous() {
        return Err(ApiError::invalid("owner can't be anonymous"));
    }
    if let Some(existing_record) = AAA_REGISTRY.with_borrow(|m| m.get(&args.canister_id)) {
        if existing_record.status == AaaStatus::Active {
            let (v, _, blob) = latest_approved_wasm()
                .ok_or_else(|| ApiError::Internal("no approved wasm available".into()))?;
            return Ok((existing_record.name, v, blob, true));
        }
        if existing_record.status == AaaStatus::Installing {
            let (v, _, blob) = latest_approved_wasm()
                .ok_or_else(|| ApiError::Internal("no approved wasm available".into()))?;
            return Ok((existing_record.name, v, blob, false));
        }
        return Err(ApiError::Conflict(
            "canister already registered with inactive status".into(),
        ));
    }
    if let Some(existing_aaa) = AAA_OWNERS.with_borrow(|m| m.get(&args.owner)) {
        if existing_aaa != args.canister_id {
            if let Some(other_record) = AAA_REGISTRY.with_borrow(|m| m.get(&existing_aaa)) {
                if other_record.status != AaaStatus::Deleted {
                    return Err(ApiError::Conflict("owner already has an active AAA".into()));
                }
            }
        }
    }

    let resolved_name = resolve_name_for_spawn(&args.name, &args.canister_id)?;
    let name_k = sc_types::limits::name_key(&resolved_name);
    AAA_NAMES.with_borrow_mut(|m| m.insert(name_k, args.canister_id));

    let (version, _, wasm_blob) = latest_approved_wasm()
        .ok_or_else(|| ApiError::Internal("no approved wasm available".into()))?;

    let record = AaaRecord {
        v: 1,
        owner: args.owner,
        name: resolved_name.clone(),
        avatar_seed: args.avatar_seed,
        wasm_version: version,
        status: AaaStatus::Installing,
        created_at: now,
        last_seen_at: now,
        last_cycles: 0,
        platform_is_controller: true,
        verified_at: 0,
        install_attempts: 1,
    };
    AAA_REGISTRY.with_borrow_mut(|m| m.insert(args.canister_id, record));
    AAA_OWNERS.with_borrow_mut(|m| m.insert(args.owner, args.canister_id));

    Ok((resolved_name, version, wasm_blob, false))
}

pub fn complete_register_aaa(
    canister_id: Principal,
    version: u32,
    module_hash: Vec<u8>,
    controllers: &[Principal],
    platform_id: Principal,
    now: u64,
) -> Result<(), ApiError> {
    if !controllers.contains(&platform_id) {
        return Err(ApiError::invalid("platform is not a controller"));
    }
    if !is_approved_module_hash(&module_hash) {
        return Err(ApiError::invalid("module hash not approved"));
    }
    let (name, owner) = AAA_REGISTRY.with_borrow_mut(|m| match m.get(&canister_id) {
        Some(mut rec) => {
            rec.status = AaaStatus::Active;
            rec.platform_is_controller = true;
            rec.verified_at = now;
            rec.wasm_version = version;
            m.insert(canister_id, rec.clone());
            Ok((rec.name, rec.owner))
        }
        None => Err(ApiError::NotFound),
    })?;
    AAA_PROVENANCE.with_borrow_mut(|m| {
        m.insert(
            canister_id,
            Provenance {
                module_hash,
                total_num_changes: 1,
                checked_at: now,
            },
        )
    });
    crate::events::record_event(
        now,
        canister_id,
        owner,
        crate::events::EventKind::AaaSpawned { name },
    );
    Ok(())
}

pub fn pre_upgrade_aaa(
    aaa: Principal,
    caller: Principal,
) -> Result<(AaaRecord, u32, Vec<u8>), ApiError> {
    let record = AAA_REGISTRY
        .with_borrow(|m| m.get(&aaa))
        .ok_or(ApiError::NotFound)?;
    if caller != record.owner {
        return Err(ApiError::Unauthorized);
    }
    if record.status != AaaStatus::Active {
        return Err(ApiError::invalid("AAA is not active"));
    }
    if !record.platform_is_controller {
        return Err(ApiError::invalid("platform is not a controller"));
    }
    let (version, _, blob) = latest_approved_wasm()
        .ok_or_else(|| ApiError::Internal("no approved wasm available".into()))?;
    Ok((record, version, blob))
}

pub fn complete_upgrade_aaa(
    aaa: Principal,
    version: u32,
    module_hash: Vec<u8>,
    total_num_changes: u64,
    now: u64,
) -> Result<(), ApiError> {
    if !is_approved_module_hash(&module_hash) {
        AAA_REGISTRY.with_borrow_mut(|m| {
            if let Some(mut rec) = m.get(&aaa) {
                rec.status = AaaStatus::Suspended;
                m.insert(aaa, rec);
            }
        });
        return Err(ApiError::Suspended);
    }
    AAA_REGISTRY.with_borrow_mut(|m| match m.get(&aaa) {
        Some(mut rec) => {
            rec.wasm_version = version;
            rec.verified_at = now;
            m.insert(aaa, rec);
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })?;
    AAA_PROVENANCE.with_borrow_mut(|m| {
        m.insert(
            aaa,
            Provenance {
                module_hash,
                total_num_changes,
                checked_at: now,
            },
        )
    });
    Ok(())
}

pub fn verify_provenance(
    aaa: Principal,
    module_hash: Option<Vec<u8>>,
    total_num_changes: u64,
    controllers: &[Principal],
    platform_id: Principal,
    now: u64,
) -> Result<(), ApiError> {
    let mut record = AAA_REGISTRY
        .with_borrow(|m| m.get(&aaa))
        .ok_or(ApiError::NotFound)?;
    record.platform_is_controller = controllers.contains(&platform_id);
    let Some(hash) = module_hash else {
        record.status = AaaStatus::Deleted;
        AAA_REGISTRY.with_borrow_mut(|m| m.insert(aaa, record));
        return Err(ApiError::Suspended);
    };
    if is_approved_module_hash(&hash) {
        if record.status == AaaStatus::Suspended || record.status == AaaStatus::Installing {
            record.status = AaaStatus::Active;
        }
        record.verified_at = now;
        AAA_REGISTRY.with_borrow_mut(|m| m.insert(aaa, record));
        AAA_PROVENANCE.with_borrow_mut(|m| {
            m.insert(
                aaa,
                Provenance {
                    module_hash: hash,
                    total_num_changes,
                    checked_at: now,
                },
            )
        });
        Ok(())
    } else {
        record.status = AaaStatus::Suspended;
        AAA_REGISTRY.with_borrow_mut(|m| m.insert(aaa, record));
        Err(ApiError::Suspended)
    }
}

pub fn record_heartbeat(
    caller: Principal,
    args: Heartbeat,
    min_interval_secs: u64,
    now_ns: u64,
) -> Result<(), ApiError> {
    let mut record = AAA_REGISTRY
        .with_borrow(|m| m.get(&caller))
        .ok_or(ApiError::NotRegistered)?;
    if record.status == AaaStatus::Suspended {
        return Err(ApiError::Suspended);
    }
    let now_secs = now_ns / 1_000_000_000;
    let last_secs = record.last_seen_at / 1_000_000_000;
    if now_secs.saturating_sub(last_secs) < min_interval_secs && record.last_cycles > 0 {
        return Ok(());
    }
    record.last_seen_at = now_ns;
    record.last_cycles = args.cycles;
    if args.wasm_version > 0 {
        record.wasm_version = args.wasm_version;
    }
    AAA_REGISTRY.with_borrow_mut(|m| m.insert(caller, record));
    Ok(())
}

pub fn record_sync_operators(
    caller: Principal,
    args: OperatorSetInput,
    now_ns: u64,
) -> Result<(), ApiError> {
    let record = AAA_REGISTRY
        .with_borrow(|m| m.get(&caller))
        .ok_or(ApiError::NotRegistered)?;
    if record.status == AaaStatus::Suspended {
        return Err(ApiError::Suspended);
    }
    AAA_OPERATORS.with_borrow_mut(|m| {
        m.insert(
            caller,
            OperatorSet {
                owner: record.owner,
                operators: args.operators,
                synced_at: now_ns,
            },
        )
    });
    Ok(())
}

pub fn record_update_profile(
    caller: Principal,
    args: UpdateAaaProfileArgs,
) -> Result<(), ApiError> {
    let mut record = AAA_REGISTRY
        .with_borrow(|m| m.get(&caller))
        .ok_or(ApiError::NotRegistered)?;
    if record.status == AaaStatus::Suspended {
        return Err(ApiError::Suspended);
    }
    if let Some(new_name) = args.name {
        let valid_name = sc_types::limits::aaa_name(&new_name)?;
        let new_key = sc_types::limits::name_key(&valid_name);
        let existing = AAA_NAMES.with_borrow(|m| m.get(&new_key));
        if let Some(other) = existing {
            if other != caller {
                return Err(ApiError::Conflict("name is taken".into()));
            }
        }
        let old_key = sc_types::limits::name_key(&record.name);
        AAA_NAMES.with_borrow_mut(|m| {
            m.remove(&old_key);
            m.insert(new_key, caller);
        });
        record.name = valid_name;
    }
    if let Some(seed) = args.avatar_seed {
        record.avatar_seed = seed;
    }
    AAA_REGISTRY.with_borrow_mut(|m| m.insert(caller, record));
    Ok(())
}

pub fn get_aaa(aaa: &Principal) -> Option<AaaRecord> {
    AAA_REGISTRY.with_borrow(|m| m.get(aaa))
}

pub fn get_aaa_by_owner(owner: &Principal) -> Option<Principal> {
    AAA_OWNERS.with_borrow(|m| m.get(owner))
}

pub fn get_aaa_owner(aaa: &Principal) -> Option<Principal> {
    AAA_REGISTRY.with_borrow(|m| m.get(aaa).map(|r| r.owner))
}

pub fn get_aaa_by_name(name: &str) -> Option<AaaRecord> {
    let key = sc_types::limits::name_key(name);
    let principal = AAA_NAMES.with_borrow(|m| m.get(&key))?;
    get_aaa(&principal)
}

pub fn list_aaas(filter: &AdminListAaasFilter, cursor: Option<u64>, limit: u32) -> Vec<AaaRecord> {
    let cap = (limit as usize).min(100);
    let skip = cursor.unwrap_or(0) as usize;
    AAA_REGISTRY.with_borrow(|m| {
        m.iter()
            .map(|e| e.value())
            .filter(|r| {
                if let Some(status) = filter.status {
                    if r.status != status {
                        return false;
                    }
                }
                if let Some(owner) = filter.owner {
                    if r.owner != owner {
                        return false;
                    }
                }
                if let Some(ref prefix) = filter.name_prefix {
                    if !r.name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .skip(skip)
            .take(cap)
            .collect()
    })
}

pub fn get_operators(aaa: &Principal) -> Option<OperatorSet> {
    AAA_OPERATORS.with_borrow(|m| m.get(aaa))
}

pub fn active_aaas_count() -> u64 {
    AAA_REGISTRY.with_borrow(|m| {
        m.iter()
            .filter(|e| e.value().status == AaaStatus::Active)
            .count() as u64
    })
}

pub fn get_provenance(aaa: &Principal) -> Option<Provenance> {
    AAA_PROVENANCE.with_borrow(|m| m.get(aaa))
}

pub fn suspend_aaa(aaa: Principal) -> Result<(), ApiError> {
    AAA_REGISTRY.with_borrow_mut(|m| match m.get(&aaa) {
        Some(mut rec) => {
            rec.status = AaaStatus::Suspended;
            m.insert(aaa, rec);
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })
}

pub fn unsuspend_aaa(aaa: Principal) -> Result<(), ApiError> {
    AAA_REGISTRY.with_borrow_mut(|m| match m.get(&aaa) {
        Some(mut rec) => {
            rec.status = AaaStatus::Active;
            m.insert(aaa, rec);
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })
}

pub fn rename_aaa(aaa: Principal, new_name: String) -> Result<(), ApiError> {
    let valid_name = sc_types::limits::aaa_name(&new_name)?;
    let new_key = sc_types::limits::name_key(&valid_name);
    let existing = AAA_NAMES.with_borrow(|m| m.get(&new_key));
    if let Some(other) = existing {
        if other != aaa {
            return Err(ApiError::Conflict("name is taken".into()));
        }
    }
    AAA_REGISTRY.with_borrow_mut(|m| match m.get(&aaa) {
        Some(mut rec) => {
            let old_key = sc_types::limits::name_key(&rec.name);
            AAA_NAMES.with_borrow_mut(|n| {
                n.remove(&old_key);
                n.insert(new_key, aaa);
            });
            rec.name = valid_name;
            m.insert(aaa, rec);
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })
}

pub fn increment_install_attempts(canister_id: &Principal) -> Result<u8, ApiError> {
    AAA_REGISTRY.with_borrow_mut(|m| match m.get(canister_id) {
        Some(mut rec) => {
            if rec.status != AaaStatus::Installing {
                return Err(ApiError::invalid("AAA is not in Installing status"));
            }
            rec.install_attempts = rec.install_attempts.saturating_add(1);
            let attempts = rec.install_attempts;
            m.insert(*canister_id, rec);
            Ok(attempts)
        }
        None => Err(ApiError::NotFound),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(b: u8) -> Principal {
        Principal::from_slice(&[b; 29])
    }

    #[test]
    fn t2_2_wasm_upload_approve_and_lookup() {
        let blob = vec![1, 2, 3, 4, 5];
        let hash = Sha256::digest(&blob).to_vec();
        assert!(matches!(
            upload_wasm(0, blob.clone(), hash.clone()),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            upload_wasm(1, vec![], hash.clone()),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            upload_wasm(1, vec![0; MAX_WASM_BYTES + 1], hash.clone()),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            upload_wasm(1, blob.clone(), vec![0; 32]),
            Err(ApiError::InvalidInput(_))
        ));

        upload_wasm(1, blob.clone(), hash.clone()).unwrap();
        assert_eq!(get_wasm(1), Some(blob));
        let meta = get_wasm_meta(1).unwrap();
        assert_eq!(meta.sha256, hash);
        assert!(!meta.approved);

        assert_eq!(approve_wasm(99, 100), Err(ApiError::NotFound));
        approve_wasm(1, 100).unwrap();
        let meta = get_wasm_meta(1).unwrap();
        assert!(meta.approved);
        assert_eq!(meta.released_at, 100);

        assert!(is_approved_module_hash(&hash));
        assert!(!is_approved_module_hash(&[9; 32]));

        let (v, m, _) = latest_approved_wasm().unwrap();
        assert_eq!(v, 1);
        assert_eq!(m.sha256, hash);
    }

    #[test]
    fn t2_2_check_name_and_suffix_resolution() {
        assert_eq!(check_name("ab"), CheckNameResult::Invalid);
        assert_eq!(check_name("ValidName_1"), CheckNameResult::Ok);

        let c1 = p(1);
        let c2 = p(2);
        let resolved = resolve_name_for_spawn("Surveyor-01", &c1).unwrap();
        assert_eq!(resolved, "Surveyor-01");

        AAA_NAMES.with_borrow_mut(|m| m.insert(sc_types::limits::name_key("Surveyor-01"), c1));
        assert_eq!(check_name("Surveyor-01"), CheckNameResult::Taken);

        assert_eq!(
            resolve_name_for_spawn("Surveyor-01", &c1).unwrap(),
            "Surveyor-01"
        );
        assert_eq!(
            resolve_name_for_spawn("Surveyor-01", &c2).unwrap(),
            "Surveyor-01-2"
        );
    }

    #[test]
    fn t2_2_registration_lifecycle_and_owner_conflict() {
        let blob = vec![1, 2, 3, 4];
        let hash = Sha256::digest(&blob).to_vec();
        upload_wasm(10, blob, hash.clone()).unwrap();
        approve_wasm(10, 50).unwrap();

        let owner = p(10);
        let canister = p(11);
        let platform = p(12);

        let bad_args = RegisterArgs {
            canister_id: canister,
            owner: Principal::anonymous(),
            name: "TestAaa".into(),
            avatar_seed: 42,
        };
        assert!(matches!(
            pre_register_aaa(&bad_args, 100),
            Err(ApiError::InvalidInput(_))
        ));

        let args = RegisterArgs {
            canister_id: canister,
            owner,
            name: "TestAaa".into(),
            avatar_seed: 42,
        };
        let (resolved, v, _, is_active) = pre_register_aaa(&args, 100).unwrap();
        assert_eq!(resolved, "TestAaa");
        assert_eq!(v, 10);
        assert!(!is_active);

        let retry = pre_register_aaa(&args, 101).unwrap();
        assert_eq!(retry.0, "TestAaa");

        let conflict_args = RegisterArgs {
            canister_id: p(13),
            owner,
            name: "SecondAaa".into(),
            avatar_seed: 43,
        };
        assert!(matches!(
            pre_register_aaa(&conflict_args, 102),
            Err(ApiError::Conflict(_))
        ));

        assert!(matches!(
            complete_register_aaa(canister, 10, hash.clone(), &[owner], platform, 200),
            Err(ApiError::InvalidInput(_))
        ));
        assert!(matches!(
            complete_register_aaa(canister, 10, vec![0; 32], &[owner, platform], platform, 200),
            Err(ApiError::InvalidInput(_))
        ));

        complete_register_aaa(
            canister,
            10,
            hash.clone(),
            &[owner, platform],
            platform,
            200,
        )
        .unwrap();

        let rec = get_aaa(&canister).unwrap();
        assert_eq!(rec.status, AaaStatus::Active);
        assert_eq!(rec.wasm_version, 10);
        assert!(rec.platform_is_controller);
        assert_eq!(get_aaa_owner(&canister), Some(owner));
        assert_eq!(get_aaa_by_owner(&owner), Some(canister));

        let idemp = pre_register_aaa(&args, 201).unwrap();
        assert!(idemp.3);
    }

    #[test]
    fn t2_2_upgrade_provenance_and_verification() {
        let owner = p(20);
        let canister = p(21);
        let platform = p(22);

        let blob_v1 = vec![10, 11];
        let hash_v1 = Sha256::digest(&blob_v1).to_vec();
        upload_wasm(1, blob_v1, hash_v1.clone()).unwrap();
        approve_wasm(1, 10).unwrap();

        let blob_v2 = vec![20, 21];
        let hash_v2 = Sha256::digest(&blob_v2).to_vec();
        upload_wasm(2, blob_v2, hash_v2.clone()).unwrap();
        approve_wasm(2, 20).unwrap();

        let args = RegisterArgs {
            canister_id: canister,
            owner,
            name: "UpgradeAaa".into(),
            avatar_seed: 1,
        };
        pre_register_aaa(&args, 100).unwrap();
        complete_register_aaa(
            canister,
            1,
            hash_v1.clone(),
            &[owner, platform],
            platform,
            105,
        )
        .unwrap();

        assert!(matches!(
            pre_upgrade_aaa(p(99), owner),
            Err(ApiError::NotFound)
        ));
        assert!(matches!(
            pre_upgrade_aaa(canister, p(99)),
            Err(ApiError::Unauthorized)
        ));

        let (_, next_v, _) = pre_upgrade_aaa(canister, owner).unwrap();
        assert_eq!(next_v, 2);

        complete_upgrade_aaa(canister, 2, hash_v2.clone(), 2, 210).unwrap();
        let updated = get_aaa(&canister).unwrap();
        assert_eq!(updated.wasm_version, 2);

        assert!(verify_provenance(
            canister,
            Some(hash_v2),
            2,
            &[owner, platform],
            platform,
            300
        )
        .is_ok());

        assert!(matches!(
            verify_provenance(
                canister,
                Some(vec![9; 32]),
                3,
                &[owner, platform],
                platform,
                301
            ),
            Err(ApiError::Suspended)
        ));
        assert_eq!(get_aaa(&canister).unwrap().status, AaaStatus::Suspended);

        assert!(matches!(
            verify_provenance(canister, None, 4, &[owner, platform], platform, 302),
            Err(ApiError::Suspended)
        ));
        assert_eq!(get_aaa(&canister).unwrap().status, AaaStatus::Deleted);
    }

    #[test]
    fn t2_2_heartbeat_sync_operators_and_profile_update() {
        let owner = p(30);
        let canister = p(31);
        let platform = p(32);

        let blob = vec![30, 31];
        let hash = Sha256::digest(&blob).to_vec();
        upload_wasm(3, blob, hash.clone()).unwrap();
        approve_wasm(3, 10).unwrap();

        let args = RegisterArgs {
            canister_id: canister,
            owner,
            name: "HeartbeatAaa".into(),
            avatar_seed: 1,
        };
        pre_register_aaa(&args, 100).unwrap();
        complete_register_aaa(canister, 3, hash, &[owner, platform], platform, 105).unwrap();

        assert!(matches!(
            record_heartbeat(
                p(99),
                Heartbeat {
                    cycles: 100,
                    wasm_version: 3
                },
                3600,
                100_000_000_000
            ),
            Err(ApiError::NotRegistered)
        ));

        record_heartbeat(
            canister,
            Heartbeat {
                cycles: 500,
                wasm_version: 3,
            },
            3600,
            100_000_000_000,
        )
        .unwrap();
        let rec = get_aaa(&canister).unwrap();
        assert_eq!(rec.last_cycles, 500);

        record_heartbeat(
            canister,
            Heartbeat {
                cycles: 600,
                wasm_version: 3,
            },
            3600,
            101_000_000_000,
        )
        .unwrap();
        let rec = get_aaa(&canister).unwrap();
        assert_eq!(rec.last_cycles, 500);

        let op_input = OperatorSetInput {
            operators: vec![(p(33), Some(9999))],
        };
        record_sync_operators(canister, op_input, 200).unwrap();
        let ops = get_operators(&canister).unwrap();
        assert_eq!(ops.operators.len(), 1);

        record_update_profile(
            canister,
            UpdateAaaProfileArgs {
                name: Some("RenamedAaa".into()),
                avatar_seed: Some(77),
            },
        )
        .unwrap();

        let rec = get_aaa(&canister).unwrap();
        assert_eq!(rec.name, "RenamedAaa");
        assert_eq!(rec.avatar_seed, 77);
        assert_eq!(get_aaa_by_name("RenamedAaa").unwrap().name, "RenamedAaa");
        assert!(get_aaa_by_name("HeartbeatAaa").is_none());

        suspend_aaa(canister).unwrap();
        assert_eq!(get_aaa(&canister).unwrap().status, AaaStatus::Suspended);
        assert!(matches!(
            record_heartbeat(
                canister,
                Heartbeat {
                    cycles: 100,
                    wasm_version: 3
                },
                3600,
                200_000_000_000
            ),
            Err(ApiError::Suspended)
        ));
        unsuspend_aaa(canister).unwrap();
        assert_eq!(get_aaa(&canister).unwrap().status, AaaStatus::Active);

        rename_aaa(canister, "AdminRenamedAaa".into()).unwrap();
        assert_eq!(get_aaa(&canister).unwrap().name, "AdminRenamedAaa");

        assert_eq!(increment_install_attempts(&p(99)), Err(ApiError::NotFound));
        assert!(matches!(
            increment_install_attempts(&canister),
            Err(ApiError::InvalidInput(_))
        ));

        let list = list_aaas(
            &AdminListAaasFilter {
                status: Some(AaaStatus::Active),
                name_prefix: Some("Admin".into()),
                owner: Some(owner),
            },
            None,
            10,
        );
        assert_eq!(list.len(), 1);
        assert!(active_aaas_count() >= 1);
    }
}
