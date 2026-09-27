use std::cell::RefCell;
use std::ops::Bound;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::{Answer, ApiError, Protocol, SubjectRef, Task};
use serde::Deserialize;

use crate::config::Params;
use crate::memory::{self, Memory};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Subject {
    pub v: u8,
    pub ref_: SubjectRef,
    pub active: bool,
    pub gold: Option<Vec<Answer>>,
    pub tally_count: u16,
}

crate::candid_storable!(Subject);
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredProtocol(pub Protocol);

crate::candid_storable!(StoredProtocol);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Lease {
    pub v: u8,
    pub aaa: Principal,
    pub subject_id: u32,
    pub issued_at: u64,
    pub expires_at: u64,
    pub consumed_by: Option<u64>,
}

crate::candid_storable!(Lease);

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeenKey {
    pub aaa: Principal,
    pub subject_id: u32,
}

impl ic_stable_structures::Storable for SeenKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 34,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(34);
        let p_bytes = self.aaa.as_slice();
        bytes.push(p_bytes.len() as u8);
        bytes.extend_from_slice(p_bytes);
        bytes.extend_from_slice(&self.subject_id.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let len = bytes[0] as usize;
        let aaa = Principal::from_slice(&bytes[1..1 + len]);
        let subject_id = u32::from_be_bytes(bytes[1 + len..1 + len + 4].try_into().unwrap());
        SeenKey { aaa, subject_id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpenLeaseKey {
    pub aaa: Principal,
    pub task_id: u64,
}

impl ic_stable_structures::Storable for OpenLeaseKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 38,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let p_bytes = self.aaa.as_slice();
        let mut bytes = Vec::with_capacity(38);
        bytes.push(p_bytes.len() as u8);
        bytes.extend_from_slice(p_bytes);
        bytes.extend_from_slice(&self.task_id.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let len = bytes[0] as usize;
        let aaa = Principal::from_slice(&bytes[1..1 + len]);
        let task_id = u64::from_be_bytes(bytes[1 + len..9 + len].try_into().unwrap());
        OpenLeaseKey { aaa, task_id }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SubjectInput {
    pub subject: SubjectRef,
    pub gold: Option<Vec<Answer>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct AdminListSubjectsFilter {
    pub field: Option<String>,
    pub active: Option<bool>,
    pub gold: Option<bool>,
}

thread_local! {
    static SUBJECTS: RefCell<StableBTreeMap<u32, Subject, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::SUBJECTS)));
    static PROTOCOLS: RefCell<StableBTreeMap<u16, StoredProtocol, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::PROTOCOLS)));
    static LEASES: RefCell<StableBTreeMap<u64, Lease, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::LEASES)));
    static SEEN_SET: RefCell<StableBTreeMap<SeenKey, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::SEEN_SET)));
    static TASK_POOL: RefCell<StableBTreeMap<u32, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::TASK_POOL)));
    static OPEN_LEASES: RefCell<StableBTreeMap<OpenLeaseKey, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::OPEN_LEASES)));
    static POOL_CURSOR: RefCell<u32> = const { RefCell::new(0) };
    static HOURLY_RATE_LIMITS: RefCell<std::collections::HashMap<Principal, (f64, u64)>> =
        RefCell::new(std::collections::HashMap::new());
}

pub fn check_and_consume_rate_limit(
    aaa: Principal,
    max_per_hour: u32,
    now_ns: u64,
) -> Result<(), ApiError> {
    if max_per_hour == 0 {
        return Ok(());
    }
    HOURLY_RATE_LIMITS.with_borrow_mut(|map| {
        let max_tokens = max_per_hour as f64;
        let fill_rate_per_ns = max_tokens / (3_600.0 * 1_000_000_000.0);
        let entry = map.entry(aaa).or_insert((max_tokens, now_ns));
        let elapsed_ns = now_ns.saturating_sub(entry.1) as f64;
        let mut tokens = entry.0 + elapsed_ns * fill_rate_per_ns;
        if tokens > max_tokens {
            tokens = max_tokens;
        }
        if tokens < 1.0 {
            let needed = 1.0 - tokens;
            let wait_secs = (needed / (max_tokens / 3600.0)).ceil() as u32;
            return Err(ApiError::RateLimited {
                retry_after_secs: wait_secs.max(1),
            });
        }
        entry.0 = tokens - 1.0;
        entry.1 = now_ns;
        Ok(())
    })
}

pub fn validate_subject_ref(s: &SubjectRef) -> Result<(), ApiError> {
    if s.subject_id == 0 {
        return Err(ApiError::invalid("subject_id must be > 0"));
    }
    sc_types::limits::field(&s.field)?;
    sc_types::limits::sha256(&s.image_sha256)?;
    sc_types::limits::sha256(&s.dossier_sha256)?;
    if !(0.0..=360.0).contains(&s.ra_deg) || !(-90.0..=90.0).contains(&s.dec_deg) {
        return Err(ApiError::invalid("coordinates out of range"));
    }
    Ok(())
}

pub fn add_subjects(batch: Vec<SubjectInput>) -> Result<u32, ApiError> {
    if batch.is_empty() || batch.len() > sc_types::limits::ADMIN_BATCH_MAX {
        return Err(ApiError::invalid(format!(
            "batch size must be 1-{}",
            sc_types::limits::ADMIN_BATCH_MAX
        )));
    }
    for item in &batch {
        validate_subject_ref(&item.subject)?;
    }
    let mut count = 0;
    SUBJECTS.with_borrow_mut(|sub_map| {
        TASK_POOL.with_borrow_mut(|pool| {
            for item in batch {
                let id = item.subject.subject_id;
                if sub_map.contains_key(&id) {
                    continue;
                }
                count += 1;
                let is_gold = item.gold.is_some();
                let subject = Subject {
                    v: 1,
                    ref_: item.subject,
                    active: true,
                    gold: item.gold,
                    tally_count: 0,
                };
                sub_map.insert(id, subject);
                if !is_gold {
                    pool.insert(id, ());
                }
            }
        });
    });
    Ok(count)
}

pub fn set_subject_active(
    subject_id: u32,
    active: bool,
    retire_after_k: u16,
) -> Result<(), ApiError> {
    SUBJECTS.with_borrow_mut(|sub_map| match sub_map.get(&subject_id) {
        Some(mut s) => {
            s.active = active;
            sub_map.insert(subject_id, s.clone());
            TASK_POOL.with_borrow_mut(|pool| {
                if !active || s.gold.is_some() || s.tally_count >= retire_after_k {
                    pool.remove(&subject_id);
                } else {
                    pool.insert(subject_id, ());
                }
            });
            Ok(())
        }
        None => Err(ApiError::NotFound),
    })
}

pub fn get_subject(subject_id: u32) -> Option<Subject> {
    SUBJECTS.with_borrow(|m| m.get(&subject_id))
}

pub fn add_protocol(protocol: Protocol) -> Result<(), ApiError> {
    if protocol.version == 0 {
        return Err(ApiError::invalid("protocol version must be > 0"));
    }
    if protocol.questions.is_empty() {
        return Err(ApiError::invalid("protocol questions cannot be empty"));
    }
    if let Some(existing) = get_protocol(protocol.version) {
        if existing == protocol {
            return Ok(());
        }
        return Err(ApiError::Conflict(format!(
            "protocol v{} is already published",
            protocol.version
        )));
    }
    PROTOCOLS.with_borrow_mut(|m| m.insert(protocol.version, StoredProtocol(protocol)));
    Ok(())
}

pub fn get_protocol(version: u16) -> Option<Protocol> {
    PROTOCOLS.with_borrow(|m| m.get(&version).map(|sp| sp.0))
}

pub fn list_protocols() -> Vec<Protocol> {
    PROTOCOLS.with_borrow(|m| m.iter().map(|e| e.value().0).collect())
}

pub fn list_subjects(
    filter: &AdminListSubjectsFilter,
    cursor: Option<u64>,
    limit: u32,
) -> Vec<Subject> {
    let cap = (limit as usize).min(100);
    let skip = cursor.unwrap_or(0) as usize;
    SUBJECTS.with_borrow(|m| {
        m.iter()
            .map(|e| e.value())
            .filter(|s| {
                if let Some(ref field) = filter.field {
                    if &s.ref_.field != field {
                        return false;
                    }
                }
                if let Some(active) = filter.active {
                    if s.active != active {
                        return false;
                    }
                }
                if let Some(gold) = filter.gold {
                    if s.gold.is_some() != gold {
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

pub fn total_subjects_count() -> u32 {
    SUBJECTS.with_borrow(|m| m.len() as u32)
}

pub fn retired_subjects_count() -> u32 {
    SUBJECTS.with_borrow(|m| {
        m.iter()
            .filter(|e| !e.value().active && e.value().gold.is_none())
            .count() as u32
    })
}

pub fn has_seen(aaa: Principal, subject_id: u32) -> bool {
    SEEN_SET.with_borrow(|m| m.contains_key(&SeenKey { aaa, subject_id }))
}

pub fn mark_seen(aaa: Principal, subject_id: u32) {
    SEEN_SET.with_borrow_mut(|m| m.insert(SeenKey { aaa, subject_id }, ()));
}

pub fn sweep_and_count_open_leases(aaa: Principal, now_ns: u64) -> usize {
    OPEN_LEASES.with_borrow_mut(|m| {
        let range = OpenLeaseKey { aaa, task_id: 0 }..=OpenLeaseKey {
            aaa,
            task_id: u64::MAX,
        };
        let (open, expired): (Vec<_>, Vec<_>) = m
            .range(range)
            .map(|e| (*e.key(), e.value()))
            .partition(|(_, expires_at)| *expires_at > now_ns);
        for (k, _) in expired {
            m.remove(&k);
        }
        open.len()
    })
}

fn next_task_id() -> u64 {
    LEASES.with_borrow(|m| m.last_key_value().map(|(k, _)| k + 1).unwrap_or(1))
}

pub fn get_lease(task_id: u64) -> Option<Lease> {
    LEASES.with_borrow(|m| m.get(&task_id))
}

pub fn update_lease(task_id: u64, lease: Lease) {
    if lease.consumed_by.is_some() {
        OPEN_LEASES.with_borrow_mut(|m| {
            m.remove(&OpenLeaseKey {
                aaa: lease.aaa,
                task_id,
            })
        });
    }
    LEASES.with_borrow_mut(|m| m.insert(task_id, lease));
}

pub fn update_subject(subject: Subject, retire_after_k: u16) {
    let id = subject.ref_.subject_id;
    let active = subject.active;
    let is_gold = subject.gold.is_some();
    let tally_count = subject.tally_count;
    SUBJECTS.with_borrow_mut(|m| m.insert(id, subject));
    TASK_POOL.with_borrow_mut(|pool| {
        if !active || is_gold || tally_count >= retire_after_k {
            pool.remove(&id);
        } else {
            pool.insert(id, ());
        }
    });
}

pub fn issue_task(
    aaa: Principal,
    classifications_count: u32,
    params: &Params,
    current_protocol_version: u16,
    now_ns: u64,
    rand_roll: u32,
) -> Result<Task, ApiError> {
    check_and_consume_rate_limit(aaa, params.max_tasks_per_aaa_per_hour, now_ns)?;

    let open_leases = sweep_and_count_open_leases(aaa, now_ns);
    if open_leases >= params.max_open_leases_per_aaa as usize {
        return Err(ApiError::RateLimited {
            retry_after_secs: params.lease_task_secs as u32,
        });
    }

    let protocol = get_protocol(current_protocol_version)
        .ok_or_else(|| ApiError::Internal("protocol version not found".into()))?;

    let gold_prob = if classifications_count < params.calibration_tasks {
        params.calibration_gold_rate_bp
    } else {
        params.gold_rate_bp
    };
    let roll = (rand_roll % 10_000) as u16;
    let want_gold = roll < gold_prob;

    let mut selected_subject: Option<Subject> = None;

    if want_gold {
        SUBJECTS.with_borrow(|sub_map| {
            for entry in sub_map.iter() {
                let s = entry.value();
                if s.active && s.gold.is_some() && !has_seen(aaa, s.ref_.subject_id) {
                    selected_subject = Some(s);
                    break;
                }
            }
        });
    }

    if selected_subject.is_none() {
        let cursor = POOL_CURSOR.with_borrow(|c| *c);
        let pick = |k: u32| -> Option<Subject> {
            if has_seen(aaa, k) {
                return None;
            }
            get_subject(k).filter(|s| s.active && s.tally_count < params.retire_after_k)
        };
        selected_subject = TASK_POOL.with_borrow(|pool| {
            pool.range((Bound::Excluded(cursor), Bound::Unbounded))
                .chain(pool.range(..=cursor))
                .find_map(|e| pick(*e.key()))
        });
        if let Some(s) = &selected_subject {
            POOL_CURSOR.with_borrow_mut(|c| *c = s.ref_.subject_id);
        }
    }

    if selected_subject.is_none() {
        SUBJECTS.with_borrow(|sub_map| {
            for entry in sub_map.iter() {
                let s = entry.value();
                if s.active && s.gold.is_some() && !has_seen(aaa, s.ref_.subject_id) {
                    selected_subject = Some(s);
                    break;
                }
            }
        });
    }

    let subject = selected_subject.ok_or(ApiError::NotFound)?;
    let subject_id = subject.ref_.subject_id;

    mark_seen(aaa, subject_id);

    let task_id = next_task_id();
    let expires_at = now_ns.saturating_add(params.lease_task_secs.saturating_mul(1_000_000_000));
    let lease = Lease {
        v: 1,
        aaa,
        subject_id,
        issued_at: now_ns,
        expires_at,
        consumed_by: None,
    };
    LEASES.with_borrow_mut(|m| m.insert(task_id, lease));
    OPEN_LEASES.with_borrow_mut(|m| m.insert(OpenLeaseKey { aaa, task_id }, expires_at));

    Ok(Task {
        task_id,
        subject: subject.ref_,
        protocol,
        lease_expires_at_ns: expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_types::{AnswerOption, DiscoveryCategory, Question};

    fn sample_ref(id: u32) -> SubjectRef {
        SubjectRef {
            subject_id: id,
            field: "ceers".into(),
            ra_deg: 214.9,
            dec_deg: 52.8,
            image_url: format!("https://data.example.com/{id}/rgb.png"),
            image_sha256: vec![1; 32],
            dossier_url: format!("https://data.example.com/{id}/dossier.json"),
            dossier_sha256: vec![2; 32],
            data_version: 1,
        }
    }

    fn sample_protocol(v: u16) -> Protocol {
        Protocol {
            version: v,
            questions: vec![Question {
                id: "q1".into(),
                prompt: "Is it smooth?".into(),
                answers: vec![
                    AnswerOption {
                        id: "smooth".into(),
                        label: "Smooth".into(),
                        next: None,
                    },
                    AnswerOption {
                        id: "featured".into(),
                        label: "Featured".into(),
                        next: None,
                    },
                ],
            }],
            discovery_categories: vec![DiscoveryCategory {
                id: "lens".into(),
                label: "Gravitational Lens".into(),
                description: "Arcs or rings".into(),
            }],
            guidance_md: "Look closely at the image.".into(),
        }
    }

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t2_3_add_subjects_and_validation() {
        let mut bad_ref = sample_ref(0);
        assert!(matches!(
            validate_subject_ref(&bad_ref),
            Err(ApiError::InvalidInput(_))
        ));

        bad_ref.subject_id = 1;
        bad_ref.field = "invalid_field".into();
        assert!(matches!(
            validate_subject_ref(&bad_ref),
            Err(ApiError::InvalidInput(_))
        ));

        bad_ref.field = "ceers".into();
        bad_ref.ra_deg = 400.0;
        assert!(matches!(
            validate_subject_ref(&bad_ref),
            Err(ApiError::InvalidInput(_))
        ));

        bad_ref.ra_deg = 200.0;
        bad_ref.image_sha256 = vec![0; 16];
        assert!(matches!(
            validate_subject_ref(&bad_ref),
            Err(ApiError::InvalidInput(_))
        ));

        assert!(matches!(
            add_subjects(vec![]),
            Err(ApiError::InvalidInput(_))
        ));

        let batch = vec![
            SubjectInput {
                subject: sample_ref(101),
                gold: None,
            },
            SubjectInput {
                subject: sample_ref(102),
                gold: Some(vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }]),
            },
        ];
        let added = add_subjects(batch).unwrap();
        assert_eq!(added, 2);

        let s1 = get_subject(101).unwrap();
        assert_eq!(s1.ref_.subject_id, 101);
        assert!(s1.gold.is_none());
        assert!(s1.active);

        let s2 = get_subject(102).unwrap();
        assert!(s2.gold.is_some());

        assert_eq!(set_subject_active(999, false, 5), Err(ApiError::NotFound));
        set_subject_active(101, false, 5).unwrap();
        assert!(!get_subject(101).unwrap().active);
        set_subject_active(101, true, 5).unwrap();
        assert!(get_subject(101).unwrap().active);

        let list = list_subjects(
            &AdminListSubjectsFilter {
                field: Some("ceers".into()),
                active: Some(true),
                gold: Some(true),
            },
            None,
            10,
        );
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].ref_.subject_id, 102);
    }

    #[test]
    fn t2_3_protocols_management() {
        assert!(matches!(
            add_protocol(Protocol {
                version: 0,
                questions: vec![],
                discovery_categories: vec![],
                guidance_md: "".into(),
            }),
            Err(ApiError::InvalidInput(_))
        ));

        let proto = sample_protocol(1);
        add_protocol(proto.clone()).unwrap();
        assert_eq!(get_protocol(1), Some(proto.clone()));
        assert_eq!(get_protocol(99), None);
        assert!(!list_protocols().is_empty());
    }

    #[test]
    fn t2_3_issue_task_never_same_subject_twice() {
        let proto = sample_protocol(2);
        add_protocol(proto).unwrap();

        let batch = vec![
            SubjectInput {
                subject: sample_ref(201),
                gold: None,
            },
            SubjectInput {
                subject: sample_ref(202),
                gold: None,
            },
            SubjectInput {
                subject: sample_ref(203),
                gold: Some(vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }]),
            },
        ];
        add_subjects(batch).unwrap();

        let params = Params {
            lease_task_secs: 1800,
            max_open_leases_per_aaa: 5,
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            calibration_tasks: 50,
            retire_after_k: 5,
            ..Params::default()
        };

        let aaa = p(10);
        let task1 = issue_task(aaa, 100, &params, 2, 1_000_000_000, 5000).unwrap();
        let task2 = issue_task(aaa, 100, &params, 2, 2_000_000_000, 5000).unwrap();
        assert_ne!(task1.subject.subject_id, task2.subject.subject_id);

        let task3 = issue_task(aaa, 100, &params, 2, 3_000_000_000, 5000).unwrap();
        assert_eq!(task3.subject.subject_id, 203);

        let exhausted = issue_task(aaa, 100, &params, 2, 4_000_000_000, 5000);
        assert_eq!(exhausted, Err(ApiError::NotFound));

        let other_aaa = p(11);
        let other_task = issue_task(other_aaa, 100, &params, 2, 5_000_000_000, 5000).unwrap();
        assert!(
            other_task.subject.subject_id == 201
                || other_task.subject.subject_id == 202
                || other_task.subject.subject_id == 203
        );
    }

    #[test]
    fn t2_3_open_leases_rate_limit_and_expiration() {
        let proto = sample_protocol(3);
        add_protocol(proto).unwrap();

        let batch = (301..=305)
            .map(|id| SubjectInput {
                subject: sample_ref(id),
                gold: None,
            })
            .collect();
        add_subjects(batch).unwrap();

        let params = Params {
            lease_task_secs: 100,
            max_open_leases_per_aaa: 2,
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            ..Params::default()
        };

        let aaa = p(20);
        let now = 10_000_000_000;
        let t1 = issue_task(aaa, 0, &params, 3, now, 0).unwrap();
        let t2 = issue_task(aaa, 0, &params, 3, now + 1_000_000_000, 0).unwrap();

        let limited = issue_task(aaa, 0, &params, 3, now + 2_000_000_000, 0);
        assert!(matches!(limited, Err(ApiError::RateLimited { .. })));

        let future = now + 150_000_000_000;
        let t3 = issue_task(aaa, 0, &params, 3, future, 0).unwrap();
        assert_ne!(t3.subject.subject_id, t1.subject.subject_id);
        assert_ne!(t3.subject.subject_id, t2.subject.subject_id);

        let l = get_lease(t1.task_id).unwrap();
        assert_eq!(l.aaa, aaa);
        assert_eq!(l.subject_id, t1.subject.subject_id);
    }

    #[test]
    fn t2_9_open_lease_index_tracks_consumption_and_sweeps_expiry() {
        add_protocol(sample_protocol(9)).unwrap();
        add_subjects(
            (901..=906)
                .map(|id| SubjectInput {
                    subject: sample_ref(id),
                    gold: None,
                })
                .collect(),
        )
        .unwrap();
        let params = Params {
            lease_task_secs: 100,
            max_open_leases_per_aaa: 2,
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            ..Params::default()
        };
        let (aaa, other) = (p(90), p(91));
        let t1 = issue_task(aaa, 0, &params, 9, 1_000, 0).unwrap();
        issue_task(aaa, 0, &params, 9, 2_000, 0).unwrap();
        issue_task(other, 0, &params, 9, 2_000, 0).unwrap();
        assert_eq!(sweep_and_count_open_leases(aaa, 3_000), 2);
        assert!(matches!(
            issue_task(aaa, 0, &params, 9, 3_000, 0),
            Err(ApiError::RateLimited { .. })
        ));

        let mut lease = get_lease(t1.task_id).unwrap();
        assert_eq!(lease.v, 1);
        lease.consumed_by = Some(1);
        update_lease(t1.task_id, lease);
        assert_eq!(sweep_and_count_open_leases(aaa, 3_000), 1);
        issue_task(aaa, 0, &params, 9, 4_000, 0).unwrap();

        let later = 500_000_000_000;
        assert_eq!(sweep_and_count_open_leases(aaa, later), 0);
        assert_eq!(OPEN_LEASES.with_borrow(|m| m.len()), 1);
        assert_eq!(sweep_and_count_open_leases(other, later), 0);
        assert_eq!(OPEN_LEASES.with_borrow(|m| m.len()), 0);
    }

    #[test]
    fn t2_9_pool_cursor_rotates_and_wraps() {
        add_protocol(sample_protocol(8)).unwrap();
        add_subjects(
            [801, 802, 803]
                .into_iter()
                .map(|id| SubjectInput {
                    subject: sample_ref(id),
                    gold: None,
                })
                .collect(),
        )
        .unwrap();
        let params = Params {
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            max_open_leases_per_aaa: 10,
            ..Params::default()
        };
        let a = issue_task(p(80), 0, &params, 8, 1, 0).unwrap();
        let b = issue_task(p(81), 0, &params, 8, 1, 0).unwrap();
        let c = issue_task(p(82), 0, &params, 8, 1, 0).unwrap();
        let d = issue_task(p(83), 0, &params, 8, 1, 0).unwrap();
        let ids: Vec<u32> = [a, b, c, d].iter().map(|t| t.subject.subject_id).collect();
        assert_eq!(ids, vec![801, 802, 803, 801]);
    }

    #[test]
    fn t2_9_readding_subjects_keeps_progress_and_protocols_are_immutable() {
        add_subjects(vec![SubjectInput {
            subject: sample_ref(701),
            gold: None,
        }])
        .unwrap();
        let mut s = get_subject(701).unwrap();
        s.tally_count = 5;
        s.active = false;
        update_subject(s, 5);
        assert!(TASK_POOL.with_borrow(|m| !m.contains_key(&701)));

        let again = add_subjects(vec![
            SubjectInput {
                subject: sample_ref(701),
                gold: Some(vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }]),
            },
            SubjectInput {
                subject: sample_ref(702),
                gold: None,
            },
        ])
        .unwrap();
        assert_eq!(again, 1);
        let kept = get_subject(701).unwrap();
        assert_eq!(kept.tally_count, 5);
        assert!(!kept.active);
        assert!(kept.gold.is_none());
        assert!(TASK_POOL.with_borrow(|m| !m.contains_key(&701)));
        assert!(TASK_POOL.with_borrow(|m| m.contains_key(&702)));

        let v7 = sample_protocol(7);
        add_protocol(v7.clone()).unwrap();
        assert_eq!(add_protocol(v7.clone()), Ok(()));
        let mut changed = v7.clone();
        changed.guidance_md = "Different guidance.".into();
        assert!(matches!(add_protocol(changed), Err(ApiError::Conflict(_))));
        assert_eq!(get_protocol(7), Some(v7));
    }
}
