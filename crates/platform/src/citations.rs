use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_certification::{AsHashTree, HashTree, LookupResult, RbTree};
use ic_stable_structures::StableBTreeMap;
use sc_types::{SubjectRef, Vote};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::discoveries::{self, DiscoveryStatus};
use crate::memory::{self, Memory};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Credit {
    pub aaa: Principal,
    pub aaa_name_at_time: String,
    pub owner: Principal,
    pub at: u64,
    pub cycles_contributed: u128,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewerCredit {
    pub credit: Credit,
    pub vote: Vote,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Citation {
    pub v: u8,
    pub public_id: String,
    pub discovery_seq: u64,
    pub subject: SubjectRef,
    pub protocol_version: u16,
    pub category: String,
    pub rationale: String,
    pub outcome: DiscoveryStatus,
    pub created_at: u64,
    pub resolved_at: u64,
    pub discoverer: Credit,
    pub corroborators: Vec<Credit>,
    pub reviewers: Vec<ReviewerCredit>,
    pub total_cycles_contributed: u128,
    pub text: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CertifiedCitation {
    pub citation: Citation,
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub witness: Vec<u8>,
}

crate::candid_storable!(Citation);

thread_local! {
    static CITATIONS: RefCell<StableBTreeMap<u64, Citation, Memory>> =
        RefCell::new(StableBTreeMap::init(memory::get(memory::CITATIONS)));
    static TREE: RefCell<RbTree<String, Vec<u8>>> = const { RefCell::new(RbTree::new()) };
}

pub fn text(
    public_id: &str,
    category_label: &str,
    discoverer: &str,
    reviewers: &[String],
    outcome: DiscoveryStatus,
    (y, m, d): (i64, i64, i64),
) -> String {
    format!(
        "{public_id} — {category_label}. Discovered by {discoverer}; reviewed by {}. Space Compute, {outcome:?} {y:04}-{m:02}-{d:02}.",
        reviewers.join(", ")
    )
}

pub fn hash(c: &Citation) -> Vec<u8> {
    Sha256::digest(candid::encode_one(c).expect("encode citation")).to_vec()
}

pub fn insert(c: Citation) {
    let fresh = CITATIONS.with_borrow_mut(|m| {
        let fresh = !m.contains_key(&c.discovery_seq);
        if fresh {
            m.insert(c.discovery_seq, c.clone());
        }
        fresh
    });
    if fresh {
        TREE.with_borrow_mut(|t| t.insert(c.public_id.clone(), hash(&c)));
        certify();
    }
}

pub fn rebuild() {
    TREE.with_borrow_mut(|t| {
        *t = RbTree::new();
        CITATIONS.with_borrow(|m| {
            for e in m.iter() {
                let c = e.value();
                t.insert(c.public_id.clone(), hash(&c));
            }
        });
    });
    certify();
}

pub fn root_hash() -> [u8; 32] {
    TREE.with_borrow(|t| t.root_hash())
}

fn certify() {
    #[cfg(target_arch = "wasm32")]
    ic_cdk::api::certified_data_set(root_hash());
}

pub fn get(seq: u64) -> Option<Citation> {
    CITATIONS.with_borrow(|m| m.get(&seq))
}

pub fn certified(public_id: &str, certificate: Vec<u8>) -> Option<CertifiedCitation> {
    let citation = get(discoveries::get_by_public_id(public_id)?.seq)?;
    let witness = TREE
        .with_borrow(|t| serde_cbor::to_vec(&t.witness(public_id.as_bytes())))
        .expect("encode witness");
    Some(CertifiedCitation {
        citation,
        certificate,
        witness,
    })
}

#[derive(Deserialize)]
struct CertificateTree {
    tree: HashTree,
}

pub fn verify(cc: &CertifiedCitation, canister: Principal) -> Result<(), String> {
    let cert: CertificateTree =
        serde_cbor::from_slice(&cc.certificate).map_err(|e| e.to_string())?;
    let witness: HashTree = serde_cbor::from_slice(&cc.witness).map_err(|e| e.to_string())?;
    let certified = match cert.tree.lookup_path([
        &b"canister"[..],
        canister.as_slice(),
        &b"certified_data"[..],
    ]) {
        LookupResult::Found(v) => v.to_vec(),
        _ => return Err("certified_data missing from certificate".into()),
    };
    if witness.digest()[..] != certified[..] {
        return Err("witness root does not match certified_data".into());
    }
    match witness.lookup_path([cc.citation.public_id.as_bytes()]) {
        LookupResult::Found(v) if v == &hash(&cc.citation)[..] => Ok(()),
        _ => Err("citation hash not proven by witness".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_certification::{labeled, leaf, Certificate};

    fn p(n: u8) -> Principal {
        Principal::from_slice(&[n; 29])
    }

    fn credit(n: u8, cycles: u128) -> Credit {
        Credit {
            aaa: p(n),
            aaa_name_at_time: format!("Agent-{n}"),
            owner: p(100 + n),
            at: u64::from(n),
            cycles_contributed: cycles,
        }
    }

    fn citation(seq: u64, public_id: &str) -> Citation {
        Citation {
            v: 1,
            public_id: public_id.into(),
            discovery_seq: seq,
            subject: SubjectRef {
                subject_id: 7,
                field: "ceers".into(),
                ra_deg: 214.9,
                dec_deg: 52.8,
                image_url: "https://x/7.png".into(),
                image_sha256: vec![1; 32],
                dossier_url: "https://x/7.json".into(),
                dossier_sha256: vec![2; 32],
                data_version: 1,
            },
            protocol_version: 1,
            category: "lens".into(),
            rationale: "arc".into(),
            outcome: DiscoveryStatus::Confirmed,
            created_at: 1,
            resolved_at: 2,
            discoverer: credit(1, 10),
            corroborators: vec![],
            reviewers: vec![ReviewerCredit {
                credit: credit(2, 5),
                vote: Vote::Agree,
            }],
            total_cycles_contributed: 15,
            text: "t".into(),
        }
    }

    #[test]
    fn t4_5_text_matches_spec_format() {
        assert_eq!(
            text(
                "SC-2026-000001",
                "Gravitational Lens",
                "Alpha",
                &["Beta".into(), "Gamma".into()],
                DiscoveryStatus::Rejected,
                (2026, 9, 7)
            ),
            "SC-2026-000001 — Gravitational Lens. Discovered by Alpha; reviewed by Beta, Gamma. Space Compute, Rejected 2026-09-07."
        );
    }

    #[test]
    fn t4_5_insert_is_frozen_and_rebuild_reproduces_root() {
        let empty = root_hash();
        let c = citation(1, "SC-2026-000001");
        insert(c.clone());
        let root = root_hash();
        assert_ne!(root, empty);
        let mut changed = c.clone();
        changed.text = "tampered".into();
        insert(changed);
        assert_eq!(get(1), Some(c.clone()));
        assert_eq!(root_hash(), root);
        TREE.with_borrow_mut(|t| *t = RbTree::new());
        assert_eq!(root_hash(), empty);
        rebuild();
        assert_eq!(root_hash(), root);
        let w = TREE.with_borrow(|t| t.witness(b"SC-2026-000001"));
        assert_eq!(w.digest(), root);
        assert_eq!(
            w.lookup_path([b"SC-2026-000001"]),
            LookupResult::Found(&hash(&c)[..])
        );
    }

    fn certificate(canister: Principal, data: &[u8]) -> Vec<u8> {
        let cert = Certificate {
            tree: labeled(
                "canister",
                labeled(canister.as_slice(), labeled("certified_data", leaf(data))),
            ),
            signature: vec![7; 48],
            delegation: None,
        };
        serde_cbor::to_vec(&cert).unwrap()
    }

    #[test]
    fn t4_5_verify_accepts_witness_and_rejects_tampering() {
        let d = discoveries::create(discoveries::NewDiscovery {
            subject_id: 7,
            classification_id: 7,
            discoverer_aaa: p(1),
            discoverer_owner: p(101),
            discoverer_name_at_time: "Agent-1".into(),
            category: "lens".into(),
            rationale: "arc".into(),
            confidence: 80,
            fee: 1,
            needed_reviews: 3,
            created_at: 1_790_467_200_000_000_000,
            claim_ra_deg: 0.0,
            claim_dec_deg: 0.0,
        });
        let id = d.public_id.clone();
        let c = citation(d.seq, &id);
        insert(c.clone());
        let canister = p(50);
        let good = certified(&id, certificate(canister, &root_hash())).unwrap();
        assert_eq!(good.citation, c);
        assert_eq!(verify(&good, canister), Ok(()));
        let mut tampered = good.clone();
        tampered.citation.text = "forged".into();
        assert!(verify(&tampered, canister).is_err());
        let stale = certified(&id, certificate(canister, &[0; 32])).unwrap();
        assert!(verify(&stale, canister).is_err());
        let other = certified(&id, certificate(p(51), &root_hash())).unwrap();
        assert!(verify(&other, canister).is_err());
        let mut junk = good.clone();
        junk.witness = vec![0xff];
        assert!(verify(&junk, canister).is_err());
        junk.certificate = vec![0xff];
        assert!(verify(&junk, canister).is_err());
    }

    #[test]
    fn t4_5_certified_unknown_public_id_is_none() {
        assert_eq!(certified("SC-2026-999999", vec![1]), None);
        let w = TREE.with_borrow(|t| t.witness(b"x"));
        let back: HashTree = serde_cbor::from_slice(&serde_cbor::to_vec(&w).unwrap()).unwrap();
        assert_eq!(back.digest(), root_hash());
    }
}
