use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::{
    Answer, ApiError, ClaimOutcome, ClassificationReceipt, ClassificationSubmission, Protocol,
};
use serde::{Deserialize, Serialize};

use crate::catalog;
use crate::claims;
use crate::config::Params;
use crate::discoveries::{self, NewDiscovery};
use crate::memory::{self, Memory};
use crate::registry;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Classification {
    pub v: u8,
    pub classification_id: u64,
    pub aaa: Principal,
    pub owner: Principal,
    pub subject_id: u32,
    pub task_id: u64,
    pub answers: Vec<Answer>,
    pub observed_image_sha256: Vec<u8>,
    pub image_mismatch: bool,
    pub discovery_seq: Option<u64>,
    pub is_gold: bool,
    pub gold_score: Option<(u8, u8)>,
    pub consensus_score: Option<(u8, u8)>,
    pub fee: u128,
    pub agent_label: Option<String>,
    pub at: u64,
    pub xp_awarded: u32,
}

crate::candid_storable!(Classification);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SubjectConsensus {
    pub v: u8,
    pub subject_id: u32,
    pub consensus: Vec<(String, String)>,
    pub resolved_at: u64,
}

crate::candid_storable!(SubjectConsensus);

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubjectClassificationKey {
    pub subject_id: u32,
    pub classification_id: u64,
}

impl ic_stable_structures::Storable for SubjectClassificationKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 12,
            is_fixed_size: true,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.subject_id.to_be_bytes());
        bytes.extend_from_slice(&self.classification_id.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let subject_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let classification_id = u64::from_be_bytes(bytes[4..12].try_into().unwrap());
        SubjectClassificationKey {
            subject_id,
            classification_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AaaClassificationKey {
    pub aaa: Principal,
    pub classification_id: u64,
}

impl ic_stable_structures::Storable for AaaClassificationKey {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 38,
            is_fixed_size: false,
        };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let p_bytes = self.aaa.as_slice();
        let mut bytes = Vec::with_capacity(1 + p_bytes.len() + 8);
        bytes.push(p_bytes.len() as u8);
        bytes.extend_from_slice(p_bytes);
        bytes.extend_from_slice(&self.classification_id.to_be_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let len = bytes[0] as usize;
        let aaa = Principal::from_slice(&bytes[1..1 + len]);
        let classification_id = u64::from_be_bytes(bytes[1 + len..9 + len].try_into().unwrap());
        AaaClassificationKey {
            aaa,
            classification_id,
        }
    }
}

thread_local! {
    static CLASSIFICATIONS: RefCell<StableBTreeMap<u64, Classification, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CLASSIFICATIONS)));
    static SUBJECT_CLASSIFICATIONS: RefCell<StableBTreeMap<SubjectClassificationKey, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::SUBJECT_CLASSIFICATIONS)));
    static AAA_CLASSIFICATIONS: RefCell<StableBTreeMap<AaaClassificationKey, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AAA_CLASSIFICATIONS)));
    static CONSENSUS: RefCell<StableBTreeMap<u32, SubjectConsensus, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CONSENSUS)));
}

const FLAG_RATE_WINDOW: usize = 100;

fn recent_flagged_count(aaa: Principal, window: usize) -> u32 {
    let ids: Vec<u64> = AAA_CLASSIFICATIONS.with_borrow(|m| {
        let start = AaaClassificationKey {
            aaa,
            classification_id: 0,
        };
        let end = AaaClassificationKey {
            aaa,
            classification_id: u64::MAX,
        };
        m.range(start..=end)
            .rev()
            .take(window)
            .map(|e| e.key().classification_id)
            .collect()
    });
    CLASSIFICATIONS.with_borrow(|m| {
        ids.iter()
            .filter(|id| m.get(id).is_some_and(|c| c.discovery_seq.is_some()))
            .count() as u32
    })
}

pub fn validate_answers(protocol: &Protocol, answers: &[Answer]) -> Result<(), ApiError> {
    if protocol.questions.is_empty() {
        return Err(ApiError::Internal("protocol questions empty".into()));
    }
    if answers.is_empty() {
        return Err(ApiError::invalid("missing answers"));
    }

    let q_map: HashMap<&str, &sc_types::Question> = protocol
        .questions
        .iter()
        .map(|q| (q.id.as_str(), q))
        .collect();

    let mut current_q = &protocol.questions[0];
    let mut step_idx = 0;

    loop {
        let curr_answer = answers
            .get(step_idx)
            .ok_or_else(|| ApiError::invalid("missing answer for question in protocol path"))?;

        if curr_answer.question_id != current_q.id {
            return Err(ApiError::invalid(format!(
                "expected question '{}', got '{}'",
                current_q.id, curr_answer.question_id
            )));
        }

        let opt = current_q
            .answers
            .iter()
            .find(|a| a.id == curr_answer.answer_id)
            .ok_or_else(|| {
                ApiError::invalid(format!(
                    "invalid answer '{}' for question '{}'",
                    curr_answer.answer_id, current_q.id
                ))
            })?;

        match &opt.next {
            Some(next_id) => {
                current_q = q_map
                    .get(next_id.as_str())
                    .ok_or_else(|| ApiError::Internal("next question not found".into()))?;
                step_idx += 1;
            }
            None => {
                if step_idx + 1 != answers.len() {
                    return Err(ApiError::invalid("extra answers provided beyond leaf"));
                }
                break;
            }
        }
    }

    Ok(())
}

pub fn score_gold(gold_answers: &[Answer], submitted_answers: &[Answer]) -> (u8, u8) {
    let gold_map: HashMap<&str, &str> = gold_answers
        .iter()
        .map(|a| (a.question_id.as_str(), a.answer_id.as_str()))
        .collect();

    let mut matches = 0u8;
    let mut compared = 0u8;

    for ans in submitted_answers {
        if let Some(&expected) = gold_map.get(ans.question_id.as_str()) {
            compared += 1;
            if ans.answer_id == expected {
                matches += 1;
            }
        }
    }

    (matches, compared)
}

fn next_classification_id() -> u64 {
    CLASSIFICATIONS.with_borrow(|m| m.last_key_value().map(|(k, _)| k + 1).unwrap_or(1))
}

pub fn get_classification(id: u64) -> Option<Classification> {
    CLASSIFICATIONS.with_borrow(|m| m.get(&id))
}

pub fn get_subject_consensus(subject_id: u32) -> Option<SubjectConsensus> {
    CONSENSUS.with_borrow(|m| m.get(&subject_id))
}

pub fn get_subject_classifications(subject_id: u32) -> Vec<Classification> {
    let keys: Vec<u64> = SUBJECT_CLASSIFICATIONS.with_borrow(|m| {
        let start = SubjectClassificationKey {
            subject_id,
            classification_id: 0,
        };
        let end = SubjectClassificationKey {
            subject_id,
            classification_id: u64::MAX,
        };
        m.range(start..=end)
            .map(|e| e.key().classification_id)
            .collect()
    });

    CLASSIFICATIONS.with_borrow(|m| keys.into_iter().filter_map(|cid| m.get(&cid)).collect())
}

pub fn classifications_count() -> u64 {
    CLASSIFICATIONS.with_borrow(|m| m.len())
}

pub fn evaluate_consensus(subject_id: u32, retired_at: u64) -> Option<SubjectConsensus> {
    let list = get_subject_classifications(subject_id);
    let valid_classifications: Vec<&Classification> =
        list.iter().filter(|c| !c.image_mismatch).collect();
    if valid_classifications.is_empty() {
        return None;
    }

    let mut q_order = Vec::new();
    let mut q_answers: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for c in &valid_classifications {
        for a in &c.answers {
            if !q_answers.contains_key(&a.question_id) {
                q_order.push(a.question_id.clone());
            }
            *q_answers
                .entry(a.question_id.clone())
                .or_default()
                .entry(a.answer_id.clone())
                .or_insert(0) += 1;
        }
    }

    let mut consensus = Vec::new();
    for q_id in q_order {
        if let Some(counts) = q_answers.get(&q_id) {
            let total_voters: u32 = counts.values().sum();
            if total_voters >= 3 {
                let mut max_count = 0;
                let mut best_answer = None;
                let mut tie = false;
                for (ans_id, &cnt) in counts {
                    if cnt > max_count {
                        max_count = cnt;
                        best_answer = Some(ans_id.clone());
                        tie = false;
                    } else if cnt == max_count {
                        tie = true;
                    }
                }
                if !tie {
                    if let Some(winner) = best_answer {
                        consensus.push((q_id, winner));
                    }
                }
            }
        }
    }

    let sub_cons = SubjectConsensus {
        v: 1,
        subject_id,
        consensus: consensus.clone(),
        resolved_at: retired_at,
    };
    CONSENSUS.with_borrow_mut(|m| m.insert(subject_id, sub_cons.clone()));

    CLASSIFICATIONS.with_borrow_mut(|m| {
        for c in &valid_classifications {
            let mut matches = 0u8;
            let mut compared = 0u8;
            for (q_id, winner) in &consensus {
                if let Some(my_ans) = c.answers.iter().find(|a| &a.question_id == q_id) {
                    compared += 1;
                    if &my_ans.answer_id == winner {
                        matches += 1;
                    }
                }
            }
            let mut updated = (*c).clone();
            updated.consensus_score = Some((matches, compared));
            m.insert(updated.classification_id, updated);
            let agree = compared > 0 && matches == compared;
            crate::events::record_event(
                retired_at,
                c.aaa,
                c.owner,
                crate::events::EventKind::ConsensusScored {
                    subject_id,
                    agree,
                    trials: compared,
                },
            );
        }
    });

    Some(sub_cons)
}

pub fn validate_submission_limits(submission: &ClassificationSubmission) -> Result<(), ApiError> {
    sc_types::limits::answers(&submission.answers)?;
    sc_types::limits::agent_label(&submission.agent_label)?;
    sc_types::limits::sha256(&submission.observed_image_sha256)?;
    submission
        .discovery
        .as_ref()
        .map_or(Ok(()), sc_types::limits::discovery_flag)
}

pub fn process_submission(
    caller: Principal,
    owner: Principal,
    submission: ClassificationSubmission,
    params: &Params,
    current_protocol_version: u16,
    now: u64,
    fee: u128,
) -> Result<ClassificationReceipt, ApiError> {
    validate_submission_limits(&submission)?;
    let mut lease = catalog::get_lease(submission.task_id).ok_or(ApiError::LeaseNotFound)?;
    if lease.aaa != caller {
        return Err(ApiError::Unauthorized);
    }
    if let Some(cid) = lease.consumed_by {
        let original = get_classification(cid)
            .ok_or_else(|| ApiError::Internal("consumed lease without classification".into()))?;
        return Ok(ClassificationReceipt {
            classification_id: cid,
            discovery_id: original
                .discovery_seq
                .and_then(discoveries::get)
                .filter(|d| d.classification_id == cid)
                .map(|d| d.public_id),
            xp_awarded: original.xp_awarded,
            duplicate: true,
            claim: None,
        });
    }
    if now > lease.expires_at {
        return Err(ApiError::LeaseExpired);
    }

    let mut subject = catalog::get_subject(lease.subject_id).ok_or(ApiError::NotFound)?;
    let protocol = catalog::get_protocol(current_protocol_version)
        .ok_or_else(|| ApiError::Internal("protocol version not found".into()))?;

    validate_answers(&protocol, &submission.answers)?;

    let image_mismatch = submission.observed_image_sha256 != subject.ref_.image_sha256;

    let is_gold = subject.gold.is_some();
    let mut gold_score = None;
    let mut consensus_score = None;
    let mut xp_awarded = 1;

    if is_gold {
        if !image_mismatch {
            let (m, c) = score_gold(subject.gold.as_ref().unwrap(), &submission.answers);
            gold_score = Some((m, c));
            if c > 0 && m == c {
                xp_awarded += 1;
            }
        }
    } else if subject.tally_count < params.retire_after_k {
        subject.tally_count += 1;
        if subject.tally_count == params.retire_after_k {
            subject.active = false;
        }
        catalog::update_subject(subject.clone(), params.retire_after_k);
    } else if let Some(cons) = get_subject_consensus(subject.ref_.subject_id) {
        if !image_mismatch {
            let mut matches = 0u8;
            let mut compared = 0u8;
            for (q_id, winner) in &cons.consensus {
                if let Some(my_ans) = submission.answers.iter().find(|a| &a.question_id == q_id) {
                    compared += 1;
                    if &my_ans.answer_id == winner {
                        matches += 1;
                    }
                }
            }
            consensus_score = Some((matches, compared));
        }
    }

    let classification_id = next_classification_id();
    lease.consumed_by = Some(classification_id);
    catalog::update_lease(submission.task_id, lease);

    let mut discovery_seq = None;
    let mut discovery_public_id = None;
    let mut claim = None;
    let mut created = false;
    if let Some(flag) = submission.discovery.as_ref() {
        let category_known = protocol
            .discovery_categories
            .iter()
            .any(|c| c.id == flag.category);
        let cap = (FLAG_RATE_WINDOW as u32 * params.max_flag_rate_bp as u32) / 10_000;
        let within_rate = recent_flagged_count(caller, FLAG_RATE_WINDOW) < cap;
        if !image_mismatch && category_known && within_rate {
            let (ra, dec) = flag
                .claim_position
                .map_or((subject.ref_.ra_deg, subject.ref_.dec_deg), |p| {
                    (p.ra_deg, p.dec_deg)
                });
            let cell = claims::cell(ra, dec, params.claim_cell_arcsec);
            let query = claims::ClaimQuery {
                field: &subject.ref_.field,
                cell,
                category: &flag.category,
                caller,
                reopen_days: params.claim_reopen_days,
                now,
                ra_deg: ra,
                dec_deg: dec,
                unique_radius_arcsec: params.claim_cell_arcsec,
            };
            match claims::resolve(&query) {
                claims::Resolution::New => {
                    let discoverer_name = registry::get_aaa(&caller)
                        .map(|r| r.name)
                        .unwrap_or_default();
                    let discovery = discoveries::create(NewDiscovery {
                        subject_id: subject.ref_.subject_id,
                        classification_id,
                        discoverer_aaa: caller,
                        discoverer_owner: owner,
                        discoverer_name_at_time: discoverer_name,
                        category: flag.category.clone(),
                        rationale: flag.rationale.clone(),
                        confidence: flag.confidence,
                        fee,
                        needed_reviews: params.reviews_min as u8,
                        created_at: now,
                        claim_ra_deg: Some(ra),
                        claim_dec_deg: Some(dec),
                    });
                    claims::index(&subject.ref_.field, cell, &flag.category, discovery.seq);
                    discovery_seq = Some(discovery.seq);
                    discovery_public_id = Some(discovery.public_id);
                    claim = Some(ClaimOutcome::New);
                    created = true;
                }
                claims::Resolution::Corroborate(existing) => {
                    claims::corroborate(
                        existing.seq,
                        claims::Corroboration {
                            aaa: caller,
                            owner,
                            classification_id,
                            at: now,
                        },
                    );
                    discovery_seq = Some(existing.seq);
                    claim = Some(ClaimOutcome::Corroborates(existing.public_id));
                }
                claims::Resolution::ClosedRecentlyRejected(rejected) => {
                    claim = Some(ClaimOutcome::ClosedRecentlyRejected(rejected.public_id));
                }
                claims::Resolution::Noop => {}
            }
        }
    }

    let classification = Classification {
        v: 1,
        classification_id,
        aaa: caller,
        owner,
        subject_id: subject.ref_.subject_id,
        task_id: submission.task_id,
        answers: submission.answers,
        observed_image_sha256: submission.observed_image_sha256,
        image_mismatch,
        discovery_seq,
        is_gold,
        gold_score,
        consensus_score,
        fee,
        agent_label: submission.agent_label,
        at: now,
        xp_awarded,
    };

    CLASSIFICATIONS.with_borrow_mut(|m| m.insert(classification_id, classification));
    SUBJECT_CLASSIFICATIONS.with_borrow_mut(|m| {
        m.insert(
            SubjectClassificationKey {
                subject_id: subject.ref_.subject_id,
                classification_id,
            },
            (),
        )
    });
    AAA_CLASSIFICATIONS.with_borrow_mut(|m| {
        m.insert(
            AaaClassificationKey {
                aaa: caller,
                classification_id,
            },
            (),
        )
    });

    if !is_gold && subject.tally_count == params.retire_after_k {
        evaluate_consensus(subject.ref_.subject_id, now);
    }

    crate::events::record_event(
        now,
        caller,
        owner,
        crate::events::EventKind::Classified {
            classification_id,
            subject_id: subject.ref_.subject_id,
            gold: gold_score,
            fee: fee as u64,
        },
    );

    if image_mismatch {
        crate::events::record_event(
            now,
            caller,
            owner,
            crate::events::EventKind::ImageMismatch {
                subject_id: subject.ref_.subject_id,
            },
        );
    }

    if let Some(seq) = discovery_seq.filter(|_| created) {
        crate::events::record_event(
            now,
            caller,
            owner,
            crate::events::EventKind::DiscoveryFlagged { seq },
        );
    }

    Ok(ClassificationReceipt {
        classification_id,
        discovery_id: discovery_public_id,
        xp_awarded,
        duplicate: false,
        claim,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_types::{AnswerOption, DiscoveryCategory, Question, SubjectRef};

    fn sample_ref(id: u32) -> SubjectRef {
        SubjectRef {
            subject_id: id,
            field: "ceers".into(),
            ra_deg: 214.9 + id as f64 * 0.001,
            dec_deg: 52.8 + id as f64 * 0.001,
            image_url: format!("https://data.example.com/{id}/rgb.png"),
            image_sha256: vec![1; 32],
            dossier_url: format!("https://data.example.com/{id}/dossier.json"),
            dossier_sha256: vec![2; 32],
            data_version: 1,
        }
    }

    fn sample_tree_protocol() -> Protocol {
        Protocol {
            version: 1,
            questions: vec![
                Question {
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
                            next: Some("q2".into()),
                        },
                    ],
                },
                Question {
                    id: "q2".into(),
                    prompt: "Has spiral arms?".into(),
                    answers: vec![
                        AnswerOption {
                            id: "yes".into(),
                            label: "Yes".into(),
                            next: None,
                        },
                        AnswerOption {
                            id: "no".into(),
                            label: "No".into(),
                            next: None,
                        },
                    ],
                },
            ],
            discovery_categories: vec![DiscoveryCategory {
                id: "lens".into(),
                label: "Lens".into(),
                description: "Lens".into(),
            }],
            guidance_md: "Guide".into(),
        }
    }

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    #[test]
    fn t2_4_answer_validation_branches_and_leaves() {
        let proto = sample_tree_protocol();

        assert!(matches!(
            validate_answers(&proto, &[]),
            Err(ApiError::InvalidInput(_))
        ));

        let valid_leaf_1 = vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }];
        assert!(validate_answers(&proto, &valid_leaf_1).is_ok());

        let extra_answers = vec![
            Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            },
            Answer {
                question_id: "q2".into(),
                answer_id: "yes".into(),
            },
        ];
        assert!(matches!(
            validate_answers(&proto, &extra_answers),
            Err(ApiError::InvalidInput(_))
        ));

        let incomplete_branch = vec![Answer {
            question_id: "q1".into(),
            answer_id: "featured".into(),
        }];
        assert!(matches!(
            validate_answers(&proto, &incomplete_branch),
            Err(ApiError::InvalidInput(_))
        ));

        let valid_branch_path = vec![
            Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            },
            Answer {
                question_id: "q2".into(),
                answer_id: "yes".into(),
            },
        ];
        assert!(validate_answers(&proto, &valid_branch_path).is_ok());

        let invalid_choice = vec![Answer {
            question_id: "q1".into(),
            answer_id: "unknown".into(),
        }];
        assert!(matches!(
            validate_answers(&proto, &invalid_choice),
            Err(ApiError::InvalidInput(_))
        ));

        let wrong_start_q = vec![Answer {
            question_id: "q2".into(),
            answer_id: "yes".into(),
        }];
        assert!(matches!(
            validate_answers(&proto, &wrong_start_q),
            Err(ApiError::InvalidInput(_))
        ));
    }

    #[test]
    fn t2_4_gold_scoring_accuracy() {
        let gold = vec![
            Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            },
            Answer {
                question_id: "q2".into(),
                answer_id: "yes".into(),
            },
        ];

        let submitted_perfect = vec![
            Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            },
            Answer {
                question_id: "q2".into(),
                answer_id: "yes".into(),
            },
        ];
        assert_eq!(score_gold(&gold, &submitted_perfect), (2, 2));

        let submitted_half = vec![
            Answer {
                question_id: "q1".into(),
                answer_id: "featured".into(),
            },
            Answer {
                question_id: "q2".into(),
                answer_id: "no".into(),
            },
        ];
        assert_eq!(score_gold(&gold, &submitted_half), (1, 2));

        let submitted_short = vec![Answer {
            question_id: "q1".into(),
            answer_id: "smooth".into(),
        }];
        assert_eq!(score_gold(&gold, &submitted_short), (0, 1));
    }

    #[test]
    fn t2_4_submission_retirement_at_k_and_consensus_and_idempotency() {
        let proto = sample_tree_protocol();
        catalog::add_protocol(proto).unwrap();

        let s_id = 9001;
        catalog::add_subjects(vec![catalog::SubjectInput {
            subject: sample_ref(s_id),
            gold: None,
        }])
        .unwrap();

        let params = Params {
            retire_after_k: 5,
            lease_task_secs: 1000,
            max_open_leases_per_aaa: 10,
            ..Params::default()
        };

        let now = 100_000_000_000;
        let mut task_ids = Vec::new();

        for i in 1..=5 {
            let caller = p(i);
            let task = catalog::issue_task(caller, 0, &params, 1, now + i as u64, 0).unwrap();
            task_ids.push((caller, task.task_id));
        }

        let bad_lease_call = process_submission(
            p(1),
            p(1),
            ClassificationSubmission {
                task_id: 999999,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: None,
                submitted_by: p(1),
            },
            &params,
            1,
            now,
            0,
        );
        assert_eq!(bad_lease_call, Err(ApiError::LeaseNotFound));

        let other_aaa_submit = process_submission(
            p(2),
            p(2),
            ClassificationSubmission {
                task_id: task_ids[0].1,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: None,
                submitted_by: p(2),
            },
            &params,
            1,
            now,
            0,
        );
        assert_eq!(other_aaa_submit, Err(ApiError::Unauthorized));

        for (i, &(caller, tid)) in task_ids[..4].iter().enumerate() {
            let ans = if i < 3 { "smooth" } else { "featured" };
            let answers = if ans == "smooth" {
                vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }]
            } else {
                vec![
                    Answer {
                        question_id: "q1".into(),
                        answer_id: "featured".into(),
                    },
                    Answer {
                        question_id: "q2".into(),
                        answer_id: "yes".into(),
                    },
                ]
            };

            let res = process_submission(
                caller,
                caller,
                ClassificationSubmission {
                    task_id: tid,
                    answers,
                    observed_image_sha256: vec![1; 32],
                    discovery: None,
                    agent_label: None,
                    submitted_by: caller,
                },
                &params,
                1,
                now + 10,
                0,
            )
            .unwrap();

            assert!(!res.duplicate);
            assert_eq!(res.xp_awarded, 1);
            let s = catalog::get_subject(s_id).unwrap();
            assert_eq!(s.tally_count, i as u16 + 1);
            assert!(s.active);
        }

        let (caller5, tid5) = task_ids[4];
        let res5 = process_submission(
            caller5,
            caller5,
            ClassificationSubmission {
                task_id: tid5,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: None,
                submitted_by: caller5,
            },
            &params,
            1,
            now + 20,
            0,
        )
        .unwrap();

        assert!(!res5.duplicate);
        let s_after = catalog::get_subject(s_id).unwrap();
        assert_eq!(s_after.tally_count, 5);
        assert!(!s_after.active);

        let dup = process_submission(
            caller5,
            caller5,
            ClassificationSubmission {
                task_id: tid5,
                answers: vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }],
                observed_image_sha256: vec![1; 32],
                discovery: None,
                agent_label: None,
                submitted_by: caller5,
            },
            &params,
            1,
            now + 30,
            0,
        )
        .unwrap();

        assert!(dup.duplicate);
        assert_eq!(dup.classification_id, res5.classification_id);
        assert_eq!(dup.xp_awarded, res5.xp_awarded);

        let cons = get_subject_consensus(s_id).expect("consensus resolved");
        assert_eq!(cons.consensus.len(), 1);
        assert_eq!(cons.consensus[0], ("q1".into(), "smooth".into()));

        let c1 = get_classification(1).unwrap();
        assert_eq!(c1.consensus_score, Some((1, 1)));
    }

    #[test]
    fn t2_9_submission_limits_enforced_at_boundary() {
        let ok = ClassificationSubmission {
            task_id: 1,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: Some("agent".into()),
            submitted_by: p(1),
        };
        assert_eq!(validate_submission_limits(&ok), Ok(()));
        let bad = [
            ClassificationSubmission {
                answers: vec![ok.answers[0].clone(); 17],
                ..ok.clone()
            },
            ClassificationSubmission {
                agent_label: Some("x".repeat(65)),
                ..ok.clone()
            },
            ClassificationSubmission {
                observed_image_sha256: vec![1; 16],
                ..ok.clone()
            },
            ClassificationSubmission {
                discovery: Some(sc_types::DiscoveryFlag {
                    category: "lens".into(),
                    rationale: "short".into(),
                    confidence: 50,
                    claim_position: None,
                }),
                ..ok.clone()
            },
        ];
        for b in bad {
            assert!(matches!(
                validate_submission_limits(&b),
                Err(ApiError::InvalidInput(_))
            ));
            assert!(matches!(
                process_submission(p(1), p(2), b, &Params::default(), 1, 0, 0),
                Err(ApiError::InvalidInput(_))
            ));
        }
    }

    fn flagged_submission(task_id: u64, caller: Principal, flag: bool) -> ClassificationSubmission {
        ClassificationSubmission {
            task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: flag.then(|| sc_types::DiscoveryFlag {
                category: "lens".into(),
                rationale: "possible arc near the core".into(),
                confidence: 70,
                claim_position: None,
            }),
            agent_label: None,
            submitted_by: caller,
        }
    }

    #[test]
    fn t4_1_discovery_flag_creates_record_and_enforces_rolling_rate_limit() {
        let proto = sample_tree_protocol();
        catalog::add_protocol(proto).unwrap();

        let params = Params {
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            calibration_tasks: 0,
            max_open_leases_per_aaa: 5,
            ..Params::default()
        };
        let caller = p(1);
        let now = 1_790_467_200_000_000_000u64;

        let issue = |subject_id: u32, at: u64| {
            catalog::add_subjects(vec![catalog::SubjectInput {
                subject: sample_ref(subject_id),
                gold: None,
            }])
            .unwrap();
            catalog::issue_task(caller, 0, &params, 1, at, 0).unwrap()
        };

        let first_task = issue(1, now);
        let receipt = process_submission(
            caller,
            caller,
            flagged_submission(first_task.task_id, caller, true),
            &params,
            1,
            now,
            0,
        )
        .unwrap();
        let public_id = receipt
            .discovery_id
            .expect("first flag creates a discovery");
        assert_eq!(public_id, "SC-2026-000001");
        assert!(discoveries::get_by_public_id(&public_id).is_some());
        let d = discoveries::get_by_public_id(&public_id).unwrap();
        assert_eq!(d.status, crate::discoveries::DiscoveryStatus::UnderReview);
        assert_eq!(d.needed_reviews, params.reviews_min as u8);
        assert_eq!(d.classification_id, receipt.classification_id);

        for i in 2..=10u32 {
            let task = issue(i, now + i as u64);
            let r = process_submission(
                caller,
                caller,
                flagged_submission(task.task_id, caller, true),
                &params,
                1,
                now + i as u64,
                0,
            )
            .unwrap();
            assert!(
                r.discovery_id.is_some(),
                "flag {i} of 10 must still be within the 10% cap"
            );
        }

        for i in 11..=100u32 {
            let task = issue(i, now + i as u64);
            let r = process_submission(
                caller,
                caller,
                flagged_submission(task.task_id, caller, false),
                &params,
                1,
                now + i as u64,
                0,
            )
            .unwrap();
            assert!(r.discovery_id.is_none());
            assert!(!r.duplicate);
        }

        let task_101 = issue(101, now + 101);
        let r101 = process_submission(
            caller,
            caller,
            flagged_submission(task_101.task_id, caller, true),
            &params,
            1,
            now + 101,
            0,
        )
        .unwrap();
        assert!(
            r101.discovery_id.is_none(),
            "the 101st flag exceeds max_flag_rate_bp and must be dropped, not error the submission"
        );
        assert!(!r101.duplicate);
        assert_eq!(discoveries::count(), 10);
    }

    #[test]
    fn t4_1_mismatched_image_and_unknown_category_silently_drop_the_flag() {
        let proto = sample_tree_protocol();
        catalog::add_protocol(proto).unwrap();
        let params = Params::default();
        let caller = p(2);
        let now = 1_790_467_200_000_000_000u64;

        catalog::add_subjects(vec![catalog::SubjectInput {
            subject: sample_ref(9101),
            gold: None,
        }])
        .unwrap();
        let task = catalog::issue_task(caller, 0, &params, 1, now, 0).unwrap();
        let mut sub = flagged_submission(task.task_id, caller, true);
        sub.observed_image_sha256 = vec![9; 32];
        let r = process_submission(caller, caller, sub, &params, 1, now, 0).unwrap();
        assert!(r.discovery_id.is_none());
        assert!(!r.duplicate);

        catalog::add_subjects(vec![catalog::SubjectInput {
            subject: sample_ref(9102),
            gold: None,
        }])
        .unwrap();
        let task2 = catalog::issue_task(caller, 0, &params, 1, now, 0).unwrap();
        let mut sub2 = flagged_submission(task2.task_id, caller, true);
        sub2.discovery.as_mut().unwrap().category = "unknown-category".into();
        let r2 = process_submission(caller, caller, sub2, &params, 1, now, 0).unwrap();
        assert!(r2.discovery_id.is_none());
        assert!(!r2.duplicate);
        assert_eq!(discoveries::count(), 0);
    }

    #[test]
    fn t4_9_same_cell_flags_collapse_into_one_discovery_with_corroborations() {
        let mut proto = sample_tree_protocol();
        proto.discovery_categories.push(DiscoveryCategory {
            id: "merger".into(),
            label: "Merger".into(),
            description: "Merger".into(),
        });
        catalog::add_protocol(proto).unwrap();
        let params = Params {
            gold_rate_bp: 0,
            calibration_gold_rate_bp: 0,
            calibration_tasks: 0,
            ..Params::default()
        };
        let now = 1_790_467_200_000_000_000u64;
        let pos = sc_types::ClaimPosition {
            ra_deg: 150.1,
            dec_deg: 2.2,
        };
        let flag = |caller: Principal, subject_id: u32, category: &str| {
            catalog::add_subjects(vec![catalog::SubjectInput {
                subject: sample_ref(subject_id),
                gold: None,
            }])
            .unwrap();
            let task = catalog::issue_task(caller, 0, &params, 1, now, 0).unwrap();
            let mut sub = flagged_submission(task.task_id, caller, true);
            let d = sub.discovery.as_mut().unwrap();
            d.claim_position = Some(pos);
            d.category = category.into();
            process_submission(caller, caller, sub, &params, 1, now, 0).unwrap()
        };

        let first = flag(p(1), 1, "lens");
        let id = first.discovery_id.clone().unwrap();
        assert_eq!(first.claim, Some(ClaimOutcome::New));
        for (caller, subject) in [(p(2), 2), (p(3), 3)] {
            let r = flag(caller, subject, "lens");
            assert_eq!(r.discovery_id, None);
            assert_eq!(r.claim, Some(ClaimOutcome::Corroborates(id.clone())));
        }
        let repeat = flag(p(2), 4, "lens");
        assert_eq!(repeat.claim, None);
        let d = discoveries::get_by_public_id(&id).unwrap();
        let corr = claims::corroborations(d.seq);
        assert_eq!(corr.len(), 2);
        assert_eq!(corr[0].aaa, p(2));
        assert_eq!(discoveries::count(), 1);

        let other = flag(p(2), 5, "merger");
        assert_eq!(other.claim, Some(ClaimOutcome::New));
        assert_ne!(other.discovery_id, Some(id));
        assert_eq!(discoveries::count(), 2);

        let mut rejected =
            discoveries::get_by_public_id(&other.discovery_id.clone().unwrap()).unwrap();
        rejected.status = crate::discoveries::DiscoveryStatus::Rejected;
        rejected.resolved_at = Some(now);
        discoveries::put(&rejected);
        let closed = flag(p(3), 6, "merger");
        assert_eq!(
            closed.claim,
            Some(ClaimOutcome::ClosedRecentlyRejected(rejected.public_id))
        );
        assert_eq!(closed.discovery_id, None);
        assert_eq!(discoveries::count(), 2);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { failure_persistence: None, ..proptest::prelude::ProptestConfig::default() })]

        #[test]
        fn t4_8_prop_every_walked_protocol_path_validates(
            depth in 1usize..=5,
            raw in proptest::collection::vec(proptest::prelude::any::<u8>(), 64),
        ) {
            let mut bytes = raw.into_iter();
            let mut next_byte = move || bytes.next().unwrap_or(0);

            let mut questions = Vec::new();
            for i in 0..depth {
                let width = 1 + (next_byte() % 3) as usize;
                let is_last = i + 1 == depth;
                let answers: Vec<AnswerOption> = (0..width)
                    .map(|a| {
                        let continues = !is_last && next_byte() % 2 == 0;
                        AnswerOption {
                            id: format!("a{a}"),
                            label: format!("Answer {a}"),
                            next: continues.then(|| format!("q{}", i + 1)),
                        }
                    })
                    .collect();
                questions.push(Question {
                    id: format!("q{i}"),
                    prompt: format!("Q{i}"),
                    answers,
                });
            }
            let proto = Protocol {
                version: 1,
                questions,
                discovery_categories: vec![],
                guidance_md: String::new(),
            };

            let mut path = Vec::new();
            let mut current = 0usize;
            loop {
                let q = &proto.questions[current];
                let choice = (next_byte() as usize) % q.answers.len();
                let opt = &q.answers[choice];
                path.push(Answer {
                    question_id: q.id.clone(),
                    answer_id: opt.id.clone(),
                });
                match &opt.next {
                    Some(next_id) => {
                        current = proto
                            .questions
                            .iter()
                            .position(|qq| &qq.id == next_id)
                            .unwrap();
                    }
                    None => break,
                }
            }

            proptest::prop_assert_eq!(validate_answers(&proto, &path), Ok(()));
        }
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    use ic_stable_structures::Memory as _;
    use sc_types::{AnswerOption, DiscoveryCategory, Question, SubjectRef};

    const MAX_BYTES_PER_CLASSIFICATION: u64 = 3 * 1024;

    type Opt = (&'static str, Option<&'static str>);

    fn protocol_v1_shape() -> Protocol {
        let spec: Vec<(&str, Vec<Opt>)> = vec![
            (
                "shape",
                vec![
                    ("smooth", Some("clumps")),
                    ("featured", Some("edgeon")),
                    ("compact", Some("odd")),
                    ("artifact", None),
                ],
            ),
            ("edgeon", vec![("yes", Some("clumps")), ("no", Some("bar"))]),
            (
                "bar",
                vec![
                    ("strong", Some("spiral")),
                    ("weak", Some("spiral")),
                    ("none", Some("spiral")),
                ],
            ),
            (
                "spiral",
                vec![("yes", Some("clumps")), ("no", Some("clumps"))],
            ),
            (
                "clumps",
                vec![
                    ("none", Some("merger")),
                    ("few", Some("merger")),
                    ("many", Some("merger")),
                ],
            ),
            (
                "merger",
                vec![
                    ("none", Some("odd")),
                    ("minor", Some("odd")),
                    ("major", Some("odd")),
                ],
            ),
            (
                "odd",
                vec![
                    ("none", None),
                    ("arc", None),
                    ("ring", None),
                    ("other", None),
                ],
            ),
        ];
        Protocol {
            version: 1,
            questions: spec
                .into_iter()
                .map(|(id, answers)| Question {
                    id: id.into(),
                    prompt: format!("{id}?"),
                    answers: answers
                        .into_iter()
                        .map(|(a, next)| AnswerOption {
                            id: a.into(),
                            label: a.into(),
                            next: next.map(Into::into),
                        })
                        .collect(),
                })
                .collect(),
            discovery_categories: vec![DiscoveryCategory {
                id: "lens".into(),
                label: "Lens".into(),
                description: "Lens".into(),
            }],
            guidance_md: "g".into(),
        }
    }

    fn walk(p: &Protocol, pick: usize) -> Vec<Answer> {
        let mut out = vec![];
        let mut q = &p.questions[0];
        loop {
            let o = &q.answers[pick % q.answers.len()];
            out.push(Answer {
                question_id: q.id.clone(),
                answer_id: o.id.clone(),
            });
            match &o.next {
                Some(n) => q = p.questions.iter().find(|x| &x.id == n).unwrap(),
                None => break,
            }
        }
        out
    }

    fn subject(id: u32) -> SubjectRef {
        SubjectRef {
            subject_id: id,
            field: "ceers".into(),
            ra_deg: 214.9,
            dec_deg: 52.8,
            image_url: format!("https://data.example.com/v1/ceers/{id}/rgb.png"),
            image_sha256: vec![1; 32],
            dossier_url: format!("https://data.example.com/v1/ceers/{id}/dossier.json"),
            dossier_sha256: vec![2; 32],
            data_version: 1,
        }
    }

    fn stable_bytes() -> u64 {
        (0..u8::MAX)
            .map(|id| crate::memory::get(id).size() * 65_536)
            .sum()
    }

    #[test]
    fn t7_12_representative_records_encode_within_bounds() {
        let p = protocol_v1_shape();
        let answers = walk(&p, 1);
        assert_eq!(answers.len(), 7);
        let aaa = Principal::from_slice(&[7; 29]);
        let c = Classification {
            v: 1,
            classification_id: u64::MAX,
            aaa,
            owner: aaa,
            subject_id: u32::MAX,
            task_id: u64::MAX,
            answers,
            observed_image_sha256: vec![1; 32],
            image_mismatch: false,
            discovery_seq: Some(u64::MAX),
            is_gold: true,
            gold_score: Some((7, 7)),
            consensus_score: Some((7, 7)),
            fee: u128::from(u64::MAX),
            agent_label: Some("x".repeat(sc_types::limits::AGENT_LABEL_MAX)),
            at: u64::MAX,
            xp_awarded: 2,
        };
        let event = crate::events::Event {
            v: 1,
            id: u64::MAX,
            at: u64::MAX,
            aaa,
            owner: aaa,
            kind: crate::events::EventKind::Classified {
                classification_id: u64::MAX,
                subject_id: u32::MAX,
                gold: Some((7, 7)),
                fee: u64::MAX,
            },
        };
        let lease = catalog::Lease {
            v: 1,
            aaa,
            subject_id: u32::MAX,
            issued_at: u64::MAX,
            expires_at: u64::MAX,
            consumed_by: Some(u64::MAX),
        };
        let len = |b: Vec<u8>| b.len();
        assert!(len(candid::encode_one(&c).unwrap()) <= 512);
        assert!(len(candid::encode_one(&event).unwrap()) <= 512);
        assert!(len(candid::encode_one(&lease).unwrap()) <= 128);
    }

    #[test]
    fn t7_12_steady_state_stable_bytes_per_classification_at_most_3_kib() {
        let p = protocol_v1_shape();
        catalog::add_protocol(p.clone()).unwrap();
        let subjects: Vec<u32> = (1..=2_000).collect();
        for chunk in subjects.chunks(sc_types::limits::ADMIN_BATCH_MAX) {
            catalog::add_subjects(
                chunk
                    .iter()
                    .map(|&id| catalog::SubjectInput {
                        subject: subject(id),
                        gold: (id % 10 == 0).then(|| walk(&p, 1)),
                    })
                    .collect(),
            )
            .unwrap();
        }
        let params = Params {
            max_open_leases_per_aaa: 10,
            max_tasks_per_aaa_per_hour: 0,
            ..Params::default()
        };
        let mut now = 1_000_000_000_000u64;
        let mut run = |rounds: std::ops::Range<u32>| {
            let mut n = 0u64;
            for round in rounds {
                for a in 0..40u8 {
                    let aaa = Principal::from_slice(&[a + 1; 29]);
                    now += 1_000_000;
                    let roll = (round * 40 + u32::from(a)).wrapping_mul(2_654_435_761);
                    let t = catalog::issue_task(aaa, round, &params, 1, now, roll).unwrap();
                    let gold = catalog::get_subject(t.subject.subject_id).unwrap().gold;
                    let submission = ClassificationSubmission {
                        task_id: t.task_id,
                        answers: gold.unwrap_or_else(|| walk(&p, usize::from(a))),
                        observed_image_sha256: vec![1; 32],
                        discovery: None,
                        agent_label: Some("claude-opus-5.5".into()),
                        submitted_by: aaa,
                    };
                    process_submission(aaa, aaa, submission, &params, 1, now, 50_000_000).unwrap();
                    n += 1;
                }
            }
            n
        };
        run(0..2);
        let before = stable_bytes();
        let n = run(2..42);
        let per = (stable_bytes() - before) / n;
        assert!(
            per <= MAX_BYTES_PER_CLASSIFICATION,
            "{per} B per classification"
        );
    }
}
