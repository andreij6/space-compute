use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use sc_types::{ApiError, ReviewAssignment, ReviewReceipt, ReviewSubmission, Vote};
use serde::{Deserialize, Serialize};

use crate::catalog;
use crate::citations::{self, Citation, Credit, ReviewerCredit};
use crate::claims;
use crate::config::Params;
use crate::credits::{self, CreditCopy, CreditOutcome, CreditRole};
use crate::discoveries::{self, Discovery, DiscoveryStatus, QUEUE_HONEYPOT};
use crate::events::{self, EventKind};
use crate::memory::{self, Memory};
use crate::progression;
use crate::registry;

pub const MAX_OPEN_ASSIGNMENTS: usize = 3;
const MIN_WEIGHT_BP: u32 = 100;
const XP_REVIEW: u32 = 3;
const XP_MATCHED: u32 = 2;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Assignment {
    pub v: u8,
    pub discovery_seq: u64,
    pub reviewer_aaa: Principal,
    pub issued_at: u64,
    pub expires_at: u64,
    pub consumed_by: Option<u64>,
}

crate::candid_storable!(Assignment);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Review {
    pub v: u8,
    pub review_id: u64,
    pub assignment_id: u64,
    pub discovery_seq: u64,
    pub reviewer_aaa: Principal,
    pub owner: Principal,
    pub vote: Vote,
    pub rationale: String,
    pub weight_bp: u32,
    pub fee: u128,
    pub observed_image_sha256: Vec<u8>,
    pub agent_label: Option<String>,
    pub at: u64,
    pub xp_awarded: u32,
}

crate::candid_storable!(Review);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct HoneypotSpec {
    pub subject_id: u32,
    pub category: String,
    pub rationale: String,
    pub truth: Vote,
}

thread_local! {
    static ASSIGNMENTS: RefCell<StableBTreeMap<u64, Assignment, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::ASSIGNMENTS)));
    static REVIEWS: RefCell<StableBTreeMap<u64, Review, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::REVIEWS)));
    static ASSIGNED: RefCell<StableBTreeMap<(Principal, u64), u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::ASSIGNED_SET)));
    static BY_DISCOVERY: RefCell<StableBTreeMap<(u64, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::DISCOVERY_ASSIGNMENTS)));
    static AWAITING: RefCell<StableBTreeMap<u64, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::AWAITING_REVIEWERS)));
}

const NS_PER_DAY: u64 = 86_400_000_000_000;
const STARVATION_MIN_REVIEWS: usize = 3;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Starvation {
    pub resolved: u32,
    pub awaiting: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Decision {
    Pending,
    Extend(u8),
    Resolve(DiscoveryStatus),
}

fn tally(votes: &[(Vote, u32)]) -> (u64, u64) {
    votes.iter().fold((0u64, 0u64), |(a, d), (vote, w)| {
        let w = u64::from((*w).max(MIN_WEIGHT_BP));
        match vote {
            Vote::Agree => (a + w, d),
            Vote::Disagree => (a, d + w),
        }
    })
}

pub fn majority(votes: &[(Vote, u32)]) -> DiscoveryStatus {
    let (a, d) = tally(votes);
    if a > d {
        DiscoveryStatus::Confirmed
    } else {
        DiscoveryStatus::Rejected
    }
}

pub fn decide(votes: &[(Vote, u32)], needed: u8, reviews_max: u16) -> Decision {
    if votes.len() < needed as usize {
        return Decision::Pending;
    }
    let (a, d) = tally(votes);
    let t = a + d;
    if a * 3 >= t * 2 {
        Decision::Resolve(DiscoveryStatus::Confirmed)
    } else if d * 2 > t {
        Decision::Resolve(DiscoveryStatus::Rejected)
    } else if u16::from(needed) < reviews_max {
        Decision::Extend(u16::from(needed).saturating_add(2).min(reviews_max) as u8)
    } else {
        Decision::Resolve(majority(votes))
    }
}

pub fn get_assignment(id: u64) -> Option<Assignment> {
    ASSIGNMENTS.with_borrow(|m| m.get(&id))
}

pub fn get_review(id: u64) -> Option<Review> {
    REVIEWS.with_borrow(|m| m.get(&id))
}

fn is_open(a: &Assignment, now: u64) -> bool {
    a.consumed_by.is_none() && now <= a.expires_at
}

fn discovery_assignments(seq: u64) -> Vec<(u64, Assignment)> {
    BY_DISCOVERY.with_borrow(|m| {
        m.range((seq, 0)..=(seq, u64::MAX))
            .filter_map(|e| {
                let id = e.key().1;
                get_assignment(id).map(|a| (id, a))
            })
            .collect()
    })
}

pub fn reviews_of(seq: u64) -> Vec<Review> {
    discovery_assignments(seq)
        .into_iter()
        .filter_map(|(_, a)| a.consumed_by.and_then(get_review))
        .collect()
}

fn taken_slots(seq: u64, now: u64) -> usize {
    discovery_assignments(seq)
        .iter()
        .filter(|(_, a)| a.consumed_by.is_some() || is_open(a, now))
        .count()
}

pub fn open_for_reviewer(aaa: Principal, now: u64) -> usize {
    ASSIGNED.with_borrow(|m| {
        m.range((aaa, 0)..=(aaa, u64::MAX))
            .filter_map(|e| get_assignment(e.value()))
            .filter(|a| is_open(a, now))
            .count()
    })
}

fn was_assigned(aaa: Principal, seq: u64) -> bool {
    ASSIGNED.with_borrow(|m| m.contains_key(&(aaa, seq)))
}

fn may_review(d: &Discovery, aaa: Principal, owner: Principal) -> bool {
    d.discoverer_aaa != aaa
        && d.discoverer_owner != owner
        && !was_assigned(aaa, d.seq)
        && !claims::corroborations(d.seq)
            .iter()
            .any(|c| c.aaa == aaa || c.owner == owner)
}

fn eligible(d: &Discovery, caller: Principal, owner: Principal, now: u64) -> bool {
    may_review(d, caller, owner) && taken_slots(d.seq, now) < d.needed_reviews as usize
}

pub fn has_eligible_reviewer(
    d: &Discovery,
    candidates: &[(Principal, Principal)],
    now: u64,
) -> bool {
    discovery_assignments(d.seq)
        .iter()
        .any(|(_, a)| is_open(a, now))
        || candidates.iter().any(|&(aaa, owner)| {
            progression::get_progress(&aaa).tier >= 2 && may_review(d, aaa, owner)
        })
}

pub fn is_awaiting_reviewers(seq: u64) -> bool {
    AWAITING.with_borrow(|m| m.contains_key(&seq))
}

pub fn apply_starvation(
    candidates: &[(Principal, Principal)],
    params: &Params,
    protocol_version: u16,
    now: u64,
) -> Starvation {
    let cutoff = now.saturating_sub(u64::from(params.review_starvation_days) * NS_PER_DAY);
    let mut out = Starvation::default();
    for d in discoveries::under_review_created_before(cutoff) {
        if has_eligible_reviewer(&d, candidates, now) {
            AWAITING.with_borrow_mut(|m| m.remove(&d.seq));
            continue;
        }
        let votes: Vec<(Vote, u32)> = reviews_of(d.seq)
            .iter()
            .map(|r| (r.vote, r.weight_bp))
            .collect();
        if votes.len() >= STARVATION_MIN_REVIEWS {
            let outcome = majority(&votes);
            resolve(d, outcome, protocol_version, now);
            out.resolved += 1;
        } else {
            AWAITING.with_borrow_mut(|m| m.insert(d.seq, now));
            out.awaiting += 1;
        }
    }
    out
}

pub fn add_honeypots(specs: Vec<HoneypotSpec>, now: u64) -> Result<u32, ApiError> {
    if specs.len() > sc_types::limits::ADMIN_BATCH_MAX {
        return Err(ApiError::invalid("too many honeypots in one batch"));
    }
    for s in &specs {
        let gold = catalog::get_subject(s.subject_id).and_then(|x| x.gold);
        if gold.is_none() {
            return Err(ApiError::invalid(format!(
                "honeypot subject {} must be an existing gold subject",
                s.subject_id
            )));
        }
        if s.category.is_empty() || s.category.len() > 64 {
            return Err(ApiError::invalid("category must be 1-64 characters"));
        }
        sc_types::limits::rationale(&s.rationale)?;
    }
    let n = specs.len() as u32;
    for s in specs {
        discoveries::create_honeypot(s.subject_id, s.category, s.rationale, s.truth, now);
    }
    Ok(n)
}

pub fn assign(
    caller: Principal,
    owner: Principal,
    params: &Params,
    protocol_version: u16,
    now: u64,
    roll: u32,
) -> Result<Option<ReviewAssignment>, ApiError> {
    if progression::get_progress(&caller).tier < 2 {
        return Err(ApiError::NotEligible("tier".into()));
    }
    if open_for_reviewer(caller, now) >= MAX_OPEN_ASSIGNMENTS {
        return Err(ApiError::NotEligible("open_assignments".into()));
    }
    let honeypot = ((roll % 10_000) as u16) < params.honeypot_rate_bp;
    let picked = honeypot
        .then(|| discoveries::find_queued(QUEUE_HONEYPOT, |d| !was_assigned(caller, d.seq)))
        .flatten()
        .or_else(|| discoveries::find_queued(0, |d| eligible(d, caller, owner, now)));
    let Some(d) = picked else {
        return Ok(None);
    };
    let subject = catalog::get_subject(d.subject_id)
        .ok_or_else(|| ApiError::Internal("discovery subject missing".into()))?;
    let expires_at = now.saturating_add(params.lease_review_secs.saturating_mul(1_000_000_000));
    let id = ASSIGNMENTS.with_borrow(|m| m.last_key_value().map_or(1, |(k, _)| k + 1));
    ASSIGNMENTS.with_borrow_mut(|m| {
        m.insert(
            id,
            Assignment {
                v: 1,
                discovery_seq: d.seq,
                reviewer_aaa: caller,
                issued_at: now,
                expires_at,
                consumed_by: None,
            },
        )
    });
    ASSIGNED.with_borrow_mut(|m| m.insert((caller, d.seq), id));
    BY_DISCOVERY.with_borrow_mut(|m| m.insert((d.seq, id), ()));
    Ok(Some(ReviewAssignment {
        assignment_id: id,
        subject: subject.ref_,
        protocol_version,
        category: d.category,
        rationale: d.rationale,
        lease_expires_at_ns: expires_at,
    }))
}

fn validate(s: &ReviewSubmission) -> Result<(), ApiError> {
    sc_types::limits::rationale(&s.rationale)?;
    sc_types::limits::agent_label(&s.agent_label)?;
    sc_types::limits::sha256(&s.observed_image_sha256)
}

pub fn submit(
    caller: Principal,
    owner: Principal,
    s: ReviewSubmission,
    params: &Params,
    protocol_version: u16,
    now: u64,
    fee: u128,
) -> Result<ReviewReceipt, ApiError> {
    validate(&s)?;
    let mut a = get_assignment(s.assignment_id).ok_or(ApiError::LeaseNotFound)?;
    if a.reviewer_aaa != caller {
        return Err(ApiError::Unauthorized);
    }
    if let Some(rid) = a.consumed_by {
        let r = get_review(rid)
            .ok_or_else(|| ApiError::Internal("consumed assignment without review".into()))?;
        return Ok(ReviewReceipt {
            review_id: rid,
            xp_awarded: r.xp_awarded,
            duplicate: true,
        });
    }
    if now > a.expires_at {
        return Err(ApiError::LeaseExpired);
    }
    let mut d = discoveries::get(a.discovery_seq).ok_or(ApiError::NotFound)?;
    if !d.is_honeypot && d.status != DiscoveryStatus::UnderReview {
        return Err(ApiError::LeaseExpired);
    }
    let review_id = REVIEWS.with_borrow(|m| m.last_key_value().map_or(1, |(k, _)| k + 1));
    let mut review = Review {
        v: 1,
        review_id,
        assignment_id: s.assignment_id,
        discovery_seq: d.seq,
        reviewer_aaa: caller,
        owner,
        vote: s.vote,
        rationale: s.rationale,
        weight_bp: progression::calculate_reputation(&progression::get_progress(&caller)),
        fee,
        observed_image_sha256: s.observed_image_sha256,
        agent_label: s.agent_label,
        at: now,
        xp_awarded: XP_REVIEW,
    };
    REVIEWS.with_borrow_mut(|m| m.insert(review_id, review.clone()));
    a.consumed_by = Some(review_id);
    ASSIGNMENTS.with_borrow_mut(|m| m.insert(s.assignment_id, a));
    events::record_event(
        now,
        caller,
        owner,
        EventKind::ReviewSubmitted {
            review_id,
            seq: d.seq,
            honeypot: d.is_honeypot,
            fee: fee as u64,
        },
    );
    let matched = if d.is_honeypot {
        let matched = d.honeypot_truth == Some(review.vote);
        events::record_event(
            now,
            caller,
            owner,
            EventKind::ReviewScored { review_id, matched },
        );
        matched
    } else {
        let votes: Vec<(Vote, u32)> = reviews_of(d.seq)
            .iter()
            .map(|r| (r.vote, r.weight_bp))
            .collect();
        match decide(&votes, d.needed_reviews, params.reviews_max) {
            Decision::Pending => false,
            Decision::Extend(needed) => {
                d.needed_reviews = needed;
                discoveries::update(&d);
                false
            }
            Decision::Resolve(outcome) => {
                resolve(d, outcome, protocol_version, now);
                review.vote == vote_of(outcome)
            }
        }
    };
    if matched {
        review.xp_awarded += XP_MATCHED;
        REVIEWS.with_borrow_mut(|m| m.insert(review_id, review.clone()));
    }
    Ok(ReviewReceipt {
        review_id,
        xp_awarded: review.xp_awarded,
        duplicate: false,
    })
}

fn vote_of(outcome: DiscoveryStatus) -> Vote {
    if outcome == DiscoveryStatus::Confirmed {
        Vote::Agree
    } else {
        Vote::Disagree
    }
}

fn name_of(aaa: &Principal) -> String {
    registry::get_aaa(aaa).map(|r| r.name).unwrap_or_default()
}

fn resolve(mut d: Discovery, outcome: DiscoveryStatus, protocol_version: u16, now: u64) {
    d.status = outcome;
    d.resolved_at = Some(now);
    discoveries::update(&d);
    AWAITING.with_borrow_mut(|m| m.remove(&d.seq));

    let reviews = reviews_of(d.seq);
    let label = catalog::get_protocol(protocol_version)
        .and_then(|p| {
            p.discovery_categories
                .into_iter()
                .find(|c| c.id == d.category)
        })
        .map_or_else(|| d.category.clone(), |c| c.label);
    let reviewer_names: Vec<String> = reviews.iter().map(|r| name_of(&r.reviewer_aaa)).collect();
    let (y, m, day) = discoveries::date_from_ns(now);
    let outcome_str = format!("{outcome:?}");
    let credit = |aaa: Principal, name: String, owner: Principal, at: u64| Credit {
        aaa,
        aaa_name_at_time: name,
        owner,
        at,
        cycles_contributed: progression::get_progress(&aaa).cycles_contributed,
    };
    let discoverer = credit(
        d.discoverer_aaa,
        d.discoverer_name_at_time.clone(),
        d.discoverer_owner,
        d.created_at,
    );
    let corroborators: Vec<Credit> = claims::corroborations(d.seq)
        .into_iter()
        .map(|c| credit(c.aaa, name_of(&c.aaa), c.owner, c.at))
        .collect();
    let reviewers: Vec<ReviewerCredit> = reviews
        .iter()
        .zip(&reviewer_names)
        .map(|(r, n)| ReviewerCredit {
            credit: credit(r.reviewer_aaa, n.clone(), r.owner, r.at),
            vote: r.vote,
        })
        .collect();
    let total_cycles_contributed = std::iter::once(&discoverer)
        .chain(&corroborators)
        .chain(reviewers.iter().map(|r| &r.credit))
        .map(|c| c.cycles_contributed)
        .sum();
    citations::insert(Citation {
        v: 1,
        public_id: d.public_id.clone(),
        discovery_seq: d.seq,
        subject: catalog::get_subject(d.subject_id).map(|s| s.ref_),
        protocol_version,
        category: d.category.clone(),
        rationale: d.rationale.clone(),
        outcome,
        created_at: d.created_at,
        resolved_at: now,
        text: format!(
            "{} — {label}. Discovered by {}; reviewed by {}. Space Compute, {outcome_str} {y:04}-{m:02}-{day:02}.",
            d.public_id,
            d.discoverer_name_at_time,
            reviewer_names.join(", ")
        ),
        discoverer,
        corroborators,
        reviewers,
        total_cycles_contributed,
    });

    events::record_event(
        now,
        d.discoverer_aaa,
        d.discoverer_owner,
        EventKind::DiscoveryResolved {
            seq: d.seq,
            outcome: outcome_str,
        },
    );
    let credit_outcome = if outcome == DiscoveryStatus::Confirmed {
        CreditOutcome::Confirmed
    } else {
        CreditOutcome::Rejected
    };
    let copy = |role| CreditCopy {
        v: 1,
        public_id: d.public_id.clone(),
        category: d.category.clone(),
        role,
        outcome: credit_outcome,
        at: now,
        subject_id: d.subject_id,
        citation_url: None,
    };
    credits::record_credit(d.discoverer_aaa, d.seq, copy(CreditRole::Discoverer));
    let truth = vote_of(outcome);
    for r in &reviews {
        events::record_event(
            now,
            r.reviewer_aaa,
            r.owner,
            EventKind::ReviewScored {
                review_id: r.review_id,
                matched: r.vote == truth,
            },
        );
        credits::record_credit(r.reviewer_aaa, d.seq, copy(CreditRole::Reviewer));
    }

    for (id, mut a) in discovery_assignments(d.seq) {
        if a.consumed_by.is_none() {
            a.expires_at = 0;
            ASSIGNMENTS.with_borrow_mut(|m| m.insert(id, a));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::SubjectInput;
    use crate::discoveries::NewDiscovery;
    use sc_types::{Answer, SubjectRef};

    const NOW: u64 = 1_790_467_200_000_000_000;
    const SEC: u64 = 1_000_000_000;

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn subject(id: u32, gold: bool) {
        catalog::add_subjects(vec![SubjectInput {
            subject: SubjectRef {
                subject_id: id,
                field: "ceers".into(),
                ra_deg: 214.9 + id as f64 * 0.001,
                dec_deg: 52.8 + id as f64 * 0.001,
                image_url: format!("https://data.example.com/{id}/rgb.png"),
                image_sha256: vec![1; 32],
                dossier_url: format!("https://data.example.com/{id}/dossier.json"),
                dossier_sha256: vec![2; 32],
                data_version: 1,
            },
            gold: gold.then(|| {
                vec![Answer {
                    question_id: "q1".into(),
                    answer_id: "smooth".into(),
                }]
            }),
        }])
        .unwrap();
    }

    fn discovery(subject_id: u32, aaa: Principal, owner: Principal, at: u64) -> Discovery {
        subject(subject_id, false);
        discoveries::create(NewDiscovery {
            subject_id,
            classification_id: subject_id as u64,
            discoverer_aaa: aaa,
            discoverer_owner: owner,
            discoverer_name_at_time: "Secret-Discoverer".into(),
            category: "lens".into(),
            rationale: "an arc around the core".into(),
            confidence: 80,
            fee: 1,
            needed_reviews: 3,
            created_at: at,
            claim_ra_deg: 0.0,
            claim_dec_deg: 0.0,
        })
    }

    fn tier2(aaa: Principal) {
        for i in 0..25 {
            events::record_event(
                NOW,
                aaa,
                aaa,
                EventKind::Classified {
                    classification_id: i,
                    subject_id: 1,
                    gold: Some((1, 1)),
                    fee: 0,
                },
            );
        }
        assert!(progression::get_progress(&aaa).tier >= 2);
    }

    fn params() -> Params {
        Params {
            honeypot_rate_bp: 0,
            ..Params::default()
        }
    }

    fn sub(id: u64, vote: Vote) -> ReviewSubmission {
        ReviewSubmission {
            assignment_id: id,
            vote,
            rationale: "the arc is clearly a lensed feature".into(),
            observed_image_sha256: vec![1; 32],
            agent_label: None,
            submitted_by: p(0),
        }
    }

    fn assign_id(aaa: Principal, owner: Principal, now: u64) -> Option<u64> {
        assign(aaa, owner, &params(), 1, now, 0)
            .unwrap()
            .map(|a| a.assignment_id)
    }

    #[test]
    fn t4_2_evaluate_decide_covers_every_branch() {
        use DiscoveryStatus::*;
        let a = |w| (Vote::Agree, w);
        let d = |w| (Vote::Disagree, w);
        assert_eq!(decide(&[a(5000), a(5000)], 3, 7), Decision::Pending);
        assert_eq!(
            decide(&[a(5000), a(5000), a(5000)], 3, 7),
            Decision::Resolve(Confirmed)
        );
        assert_eq!(
            decide(&[a(5000), a(5000), d(5000)], 3, 7),
            Decision::Resolve(Confirmed)
        );
        assert_eq!(
            decide(&[a(5000), d(5000), d(5000)], 3, 7),
            Decision::Resolve(Rejected)
        );
        let split = [a(5000), a(5000), d(5000), d(5000)];
        assert_eq!(decide(&split, 3, 7), Decision::Extend(5));
        assert_eq!(decide(&split, 6, 7), Decision::Pending);
        assert_eq!(decide(&split, 4, 5), Decision::Extend(5));
        assert_eq!(decide(&split, 4, 4), Decision::Resolve(Rejected));
        assert_eq!(
            decide(&[a(5500), d(4500)], 2, 2),
            Decision::Resolve(Confirmed)
        );
        assert_eq!(
            decide(&[a(0), d(0), d(150)], 3, 3),
            Decision::Resolve(Rejected)
        );
        assert_eq!(
            decide(&[a(0), a(0), d(9000)], 3, 7),
            Decision::Resolve(Rejected)
        );
    }

    #[test]
    fn t4_2_tier1_caller_is_not_eligible() {
        discovery(1, p(1), p(101), NOW);
        assert_eq!(
            assign(p(2), p(102), &params(), 1, NOW, 0),
            Err(ApiError::NotEligible("tier".into()))
        );
    }

    #[test]
    fn t4_2_same_owner_sibling_aaa_is_never_assigned() {
        let owner = p(100);
        let (sibling_a, sibling_b, stranger) = (p(1), p(2), p(3));
        tier2(sibling_b);
        tier2(stranger);
        let d = discovery(1, sibling_a, owner, NOW);
        for roll in [0, 5_000, 9_999] {
            assert_eq!(assign(sibling_b, owner, &params(), 1, NOW, roll), Ok(None));
        }
        assert_eq!(
            assign(sibling_a, owner, &params(), 1, NOW, 0),
            Err(ApiError::NotEligible("tier".into()))
        );
        tier2(sibling_a);
        assert_eq!(assign_id(sibling_a, owner, NOW), None);
        let id = assign_id(stranger, p(103), NOW).unwrap();
        assert_eq!(get_assignment(id).unwrap().discovery_seq, d.seq);
    }

    #[test]
    fn t4_2_assignment_is_blind_and_respects_eligibility() {
        let r = p(2);
        tier2(r);
        let d_old = discovery(1, p(1), p(101), NOW);
        let d_new = discovery(2, p(4), p(104), NOW + 1);
        claims::corroborate(
            d_old.seq,
            claims::Corroboration {
                aaa: p(5),
                owner: p(105),
                classification_id: 9,
                at: NOW,
            },
        );
        tier2(p(5));
        assert_eq!(
            assign_id(p(5), p(105), NOW).map(|id| get_assignment(id).unwrap().discovery_seq),
            Some(d_new.seq)
        );

        let a = assign(r, p(102), &params(), 7, NOW, 0).unwrap().unwrap();
        let shown = format!("{a:?}");
        for secret in [
            d_old.public_id.as_str(),
            "Secret-Discoverer",
            &p(1).to_text(),
            &p(101).to_text(),
        ] {
            assert!(!shown.contains(secret), "leaked {secret}");
        }
        assert_eq!(a.protocol_version, 7);
        assert_eq!(a.subject.subject_id, 1);
        assert_eq!(a.lease_expires_at_ns, NOW + 86_400 * SEC);
        assert_eq!(
            get_assignment(a.assignment_id).unwrap().discovery_seq,
            d_old.seq
        );

        let b = assign_id(r, p(102), NOW).unwrap();
        assert_eq!(get_assignment(b).unwrap().discovery_seq, d_new.seq);
        assert_eq!(assign_id(r, p(102), NOW), None);
        assert_eq!(open_for_reviewer(r, NOW), 2);
    }

    #[test]
    fn t4_2_slots_open_limit_and_expiry() {
        let d = discovery(1, p(1), p(101), NOW);
        for n in 2..=5 {
            tier2(p(n));
        }
        for n in 2..=4 {
            assert!(assign_id(p(n), p(100 + n), NOW).is_some());
        }
        assert_eq!(assign_id(p(5), p(105), NOW), None);
        let later = NOW + 86_401 * SEC;
        assert_eq!(
            assign_id(p(5), p(105), later).map(|id| get_assignment(id).unwrap().discovery_seq),
            Some(d.seq)
        );
        assert_eq!(assign_id(p(2), p(102), later), None);

        let busy = p(6);
        tier2(busy);
        for s in 10..13 {
            discovery(s, p(1), p(101), NOW);
        }
        for _ in 0..3 {
            assert!(assign_id(busy, p(106), later).is_some());
        }
        assert_eq!(
            assign(busy, p(106), &params(), 1, later, 0),
            Err(ApiError::NotEligible("open_assignments".into()))
        );
    }

    #[test]
    fn t4_2_submit_validates_lease_and_is_idempotent() {
        let r = p(2);
        tier2(r);
        let d = discovery(1, p(1), p(101), NOW);
        let id = assign_id(r, p(102), NOW).unwrap();
        let pr = params();
        assert_eq!(
            submit(r, p(102), sub(999, Vote::Agree), &pr, 1, NOW, 5),
            Err(ApiError::LeaseNotFound)
        );
        assert_eq!(
            submit(p(3), p(103), sub(id, Vote::Agree), &pr, 1, NOW, 5),
            Err(ApiError::Unauthorized)
        );
        let mut short = sub(id, Vote::Agree);
        short.rationale = "too short".into();
        assert!(matches!(
            submit(r, p(102), short, &pr, 1, NOW, 5),
            Err(ApiError::InvalidInput(_))
        ));
        assert_eq!(
            submit(
                r,
                p(102),
                sub(id, Vote::Agree),
                &pr,
                1,
                NOW + 86_401 * SEC,
                5
            ),
            Err(ApiError::LeaseExpired)
        );
        let weight = progression::calculate_reputation(&progression::get_progress(&r));
        let first = submit(r, p(102), sub(id, Vote::Agree), &pr, 1, NOW, 5).unwrap();
        assert!(!first.duplicate);
        assert_eq!(first.xp_awarded, 3);
        let again = submit(r, p(102), sub(id, Vote::Disagree), &pr, 1, NOW + 1, 5).unwrap();
        assert!(again.duplicate);
        assert_eq!(again.review_id, first.review_id);
        let stored = get_review(first.review_id).unwrap();
        assert_eq!(stored.weight_bp, weight);
        assert_eq!(stored.vote, Vote::Agree);
        assert_eq!(reviews_of(d.seq).len(), 1);
        assert_eq!(
            discoveries::get(d.seq).unwrap().status,
            DiscoveryStatus::UnderReview
        );
    }

    #[test]
    fn t4_2_three_agreeing_reviewers_confirm_with_citation_credits_and_expiry() {
        let d = discovery(1, p(1), p(101), NOW);
        let pr = params();
        let ids: Vec<u64> = (2..=4)
            .map(|n| {
                tier2(p(n));
                assign_id(p(n), p(100 + n), NOW).unwrap()
            })
            .collect();
        tier2(p(9));
        let late = 777;
        ASSIGNMENTS.with_borrow_mut(|m| {
            m.insert(
                late,
                Assignment {
                    v: 1,
                    discovery_seq: d.seq,
                    reviewer_aaa: p(9),
                    issued_at: NOW,
                    expires_at: NOW + SEC,
                    consumed_by: None,
                },
            )
        });
        BY_DISCOVERY.with_borrow_mut(|m| m.insert((d.seq, late), ()));
        let xp_before = progression::get_progress(&p(1)).xp;
        let mut receipts = vec![];
        for (i, id) in ids.iter().enumerate() {
            let n = i as u8 + 2;
            receipts.push(submit(p(n), p(100 + n), sub(*id, Vote::Agree), &pr, 1, NOW, 5).unwrap());
        }
        assert_eq!(receipts[0].xp_awarded, 3);
        assert_eq!(receipts[2].xp_awarded, 5);
        let resolved = discoveries::get(d.seq).unwrap();
        assert_eq!(resolved.status, DiscoveryStatus::Confirmed);
        assert_eq!(resolved.resolved_at, Some(NOW));
        assert_eq!(progression::get_progress(&p(1)).xp, xp_before + 50);
        assert_eq!(progression::get_progress(&p(2)).rev_hits, 1);
        let c = citations::get(d.seq).unwrap();
        assert_eq!(c.reviewers.len(), 3);
        assert_eq!(c.outcome, DiscoveryStatus::Confirmed);
        assert!(c
            .text
            .starts_with("SC-2026-000001 — lens. Discovered by Secret-Discoverer"));
        assert!(c.text.ends_with("Confirmed 2026-09-27."));
        assert_eq!(get_assignment(late).unwrap().expires_at, 0);
        assert_eq!(
            submit(p(9), p(109), sub(late, Vote::Agree), &pr, 1, NOW, 5),
            Err(ApiError::LeaseExpired)
        );
        let page = credits::list_aaa_credits(credits::ListAaaCreditsArgs {
            aaa: p(3),
            cursor: 0,
        });
        assert_eq!(page.items[0].role, CreditRole::Reviewer);
        assert_eq!(assign_id(p(9), p(109), NOW), None);
    }

    #[test]
    fn t4_2_two_disagreeing_reviewers_reject() {
        let d = discovery(1, p(1), p(101), NOW);
        let pr = params();
        for (n, v) in [(2, Vote::Agree), (3, Vote::Disagree), (4, Vote::Disagree)] {
            tier2(p(n));
            let id = assign_id(p(n), p(100 + n), NOW).unwrap();
            submit(p(n), p(100 + n), sub(id, v), &pr, 1, NOW, 5).unwrap();
        }
        assert_eq!(
            discoveries::get(d.seq).unwrap().status,
            DiscoveryStatus::Rejected
        );
        assert_eq!(
            citations::get(d.seq).unwrap().outcome,
            DiscoveryStatus::Rejected
        );
        assert_eq!(progression::get_progress(&p(2)).rev_hits, 0);
        assert_eq!(progression::get_progress(&p(3)).rev_hits, 1);
    }

    #[test]
    fn t4_2_split_three_way_extends_needed_reviews() {
        let d = discovery(1, p(1), p(101), NOW);
        let mut pr = params();
        pr.reviews_max = 7;
        let heavy = |aaa: Principal| {
            for i in 0..5 {
                events::record_event(
                    NOW,
                    aaa,
                    aaa,
                    EventKind::ReviewScored {
                        review_id: i,
                        matched: true,
                    },
                );
            }
        };
        let votes = [(2, Vote::Agree), (3, Vote::Disagree), (4, Vote::Agree)];
        tier2(p(3));
        heavy(p(3));
        heavy(p(3));
        for (n, v) in votes {
            if n != 3 {
                tier2(p(n));
            }
            let id = assign_id(p(n), p(100 + n), NOW).unwrap();
            submit(p(n), p(100 + n), sub(id, v), &pr, 1, NOW, 5).unwrap();
        }
        let after = discoveries::get(d.seq).unwrap();
        assert_eq!(after.status, DiscoveryStatus::UnderReview);
        assert_eq!(after.needed_reviews, 5);
    }

    #[test]
    fn t4_2_honeypots_are_assigned_by_rate_and_scored_immediately() {
        subject(50, true);
        subject(51, false);
        assert!(matches!(
            add_honeypots(
                vec![HoneypotSpec {
                    subject_id: 51,
                    category: "lens".into(),
                    rationale: "a lens on a clean elliptical".into(),
                    truth: Vote::Disagree,
                }],
                NOW
            ),
            Err(ApiError::InvalidInput(_))
        ));
        let bad_cat = HoneypotSpec {
            subject_id: 50,
            category: String::new(),
            rationale: "a lens on a clean elliptical".into(),
            truth: Vote::Disagree,
        };
        assert!(add_honeypots(vec![bad_cat.clone()], NOW).is_err());
        assert!(add_honeypots(vec![bad_cat; 501], NOW).is_err());
        let spec = HoneypotSpec {
            subject_id: 50,
            category: "lens".into(),
            rationale: "a lens on a clean elliptical".into(),
            truth: Vote::Disagree,
        };
        assert_eq!(add_honeypots(vec![spec], NOW), Ok(1));
        let d = discovery(1, p(1), p(101), NOW);
        let r = p(2);
        tier2(r);
        let mut pr = params();
        pr.honeypot_rate_bp = 1_000;
        let hp = assign(r, p(102), &pr, 1, NOW, 999).unwrap().unwrap();
        assert_eq!(hp.subject.subject_id, 50);
        assert!(
            !discoveries::get(get_assignment(hp.assignment_id).unwrap().discovery_seq)
                .unwrap()
                .public_id
                .starts_with("SC-")
        );
        let normal = assign(r, p(102), &pr, 1, NOW, 1_000).unwrap().unwrap();
        assert_eq!(normal.subject.subject_id, 1);

        let before = progression::get_progress(&r);
        let receipt = submit(
            r,
            p(102),
            sub(hp.assignment_id, Vote::Disagree),
            &pr,
            1,
            NOW,
            5,
        )
        .unwrap();
        assert_eq!(receipt.xp_awarded, 5);
        let after = progression::get_progress(&r);
        assert_eq!(after.rev_trials, before.rev_trials + 1);
        assert_eq!(after.rev_hits, before.rev_hits + 1);
        assert_eq!(
            discoveries::get(d.seq).unwrap().status,
            DiscoveryStatus::UnderReview
        );

        let r2 = p(3);
        tier2(r2);
        let hp2 = assign(r2, p(103), &pr, 1, NOW, 0).unwrap().unwrap();
        let miss = submit(
            r2,
            p(103),
            sub(hp2.assignment_id, Vote::Agree),
            &pr,
            1,
            NOW,
            5,
        )
        .unwrap();
        assert_eq!(miss.xp_awarded, 3);
        assert_eq!(progression::get_progress(&r2).rev_hits, 0);
        let fallback = assign(r2, p(103), &pr, 1, NOW, 0).unwrap().unwrap();
        assert_eq!(fallback.subject.subject_id, 1);
    }

    #[test]
    fn t4_4_resolution_path_is_synchronous_within_one_message() {
        let _: fn(_, _, _, _, _, _, _) -> Result<ReviewReceipt, ApiError> = submit;
        let _: fn(_, _, _, _) = resolve;
        let _: fn(_, _, _, _) -> Starvation = apply_starvation;
    }

    #[test]
    fn t4_4_weighted_majority_breaks_ties_to_rejected() {
        use DiscoveryStatus::*;
        let a = |w| (Vote::Agree, w);
        let d = |w| (Vote::Disagree, w);
        assert_eq!(majority(&[a(1000), a(1000), d(5000)]), Rejected);
        assert_eq!(majority(&[a(6000), d(3000), d(2000)]), Confirmed);
        assert_eq!(majority(&[a(5000), d(5000)]), Rejected);
        assert_eq!(majority(&[a(0), d(0), a(0)]), Confirmed);
    }

    #[test]
    fn t4_4_starvation_eligibility_check() {
        let d = discovery(1, p(1), p(101), NOW);
        let sibling = p(50);
        tier2(sibling);
        tier2(p(1));
        let assigned = p(3);
        tier2(assigned);
        assert!(assign_id(assigned, p(103), NOW).is_some());
        let corroborator = p(4);
        tier2(corroborator);
        claims::corroborate(
            d.seq,
            claims::Corroboration {
                aaa: corroborator,
                owner: p(104),
                classification_id: 9,
                at: NOW,
            },
        );
        let rookie = p(2);
        let blocked = [
            (p(1), p(101)),
            (sibling, p(101)),
            (rookie, p(102)),
            (assigned, p(103)),
            (corroborator, p(104)),
        ];
        assert!(has_eligible_reviewer(&d, &blocked, NOW));
        let later = NOW + 86_401 * SEC;
        assert!(!has_eligible_reviewer(&d, &blocked, later));
        tier2(p(6));
        let mut open = blocked.to_vec();
        open.push((p(6), p(106)));
        assert!(has_eligible_reviewer(&d, &open, later));
    }

    #[test]
    fn t4_4_starved_discovery_resolves_by_weighted_majority_or_awaits_reviewers() {
        let pr = params();
        let mut d1 = discovery(1, p(1), p(101), NOW);
        d1.needed_reviews = 5;
        discoveries::update(&d1);
        for (n, v) in [(2, Vote::Agree), (3, Vote::Agree), (4, Vote::Disagree)] {
            tier2(p(n));
            let id = assign_id(p(n), p(100 + n), NOW).unwrap();
            submit(p(n), p(100 + n), sub(id, v), &pr, 1, NOW, 5).unwrap();
        }
        let d2 = discovery(2, p(1), p(101), NOW + 1);
        let id = assign_id(p(2), p(102), NOW).unwrap();
        submit(p(2), p(102), sub(id, Vote::Agree), &pr, 1, NOW, 5).unwrap();
        assert_eq!(
            discoveries::get(d1.seq).unwrap().status,
            DiscoveryStatus::UnderReview
        );
        let candidates = [(p(1), p(101)), (p(2), p(102)), (p(5), p(105))];
        let xp_before = progression::get_progress(&p(1)).xp;
        let edge = NOW + 7 * 86_400 * SEC;
        assert_eq!(
            apply_starvation(&candidates, &pr, 1, edge - 1),
            Starvation::default()
        );
        assert_eq!(
            apply_starvation(&candidates, &pr, 1, edge),
            Starvation {
                resolved: 1,
                awaiting: 0
            }
        );
        let r1 = discoveries::get(d1.seq).unwrap();
        assert_eq!(r1.status, DiscoveryStatus::Confirmed);
        assert_eq!(r1.resolved_at, Some(edge));
        let c = citations::get(d1.seq).unwrap();
        assert_eq!(c.reviewers.len(), 3);
        assert_eq!(c.outcome, DiscoveryStatus::Confirmed);
        assert_eq!(progression::get_progress(&p(1)).xp, xp_before + 50);
        assert_eq!(progression::get_progress(&p(4)).rev_hits, 0);

        assert_eq!(
            apply_starvation(&candidates, &pr, 1, edge + 1),
            Starvation {
                resolved: 0,
                awaiting: 1
            }
        );
        assert!(is_awaiting_reviewers(d2.seq));
        assert_eq!(
            discoveries::get(d2.seq).unwrap().status,
            DiscoveryStatus::UnderReview
        );
        tier2(p(6));
        let mut more = candidates.to_vec();
        more.push((p(6), p(106)));
        assert_eq!(
            apply_starvation(&more, &pr, 1, edge + 2),
            Starvation::default()
        );
        assert!(!is_awaiting_reviewers(d2.seq));
        apply_starvation(&candidates, &pr, 1, edge + 3);
        assert!(is_awaiting_reviewers(d2.seq));
        for n in [6, 7] {
            tier2(p(n));
            let id = assign_id(p(n), p(100 + n), edge + 3).unwrap();
            submit(
                p(n),
                p(100 + n),
                sub(id, Vote::Disagree),
                &pr,
                1,
                edge + 3,
                5,
            )
            .unwrap();
        }
        assert_eq!(
            discoveries::get(d2.seq).unwrap().status,
            DiscoveryStatus::Rejected
        );
        assert!(!is_awaiting_reviewers(d2.seq));
    }
}
