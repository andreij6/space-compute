use candid::Principal;
use platform::events::{Event, EventKind};
use platform::progression::{apply_event, get_leaderboard, get_progress};
use platform::registry::{pre_register_aaa, RegisterArgs};
use sha2::Digest;

#[test]
fn t7_9_s16_leaderboard_excludes_aaa_below_tier2_then_admits_it_at_tier2() {
    let blob = vec![16, 1, 6];
    let _ =
        platform::registry::upload_wasm(316, blob.clone(), sha2::Sha256::digest(&blob).to_vec());
    let _ = platform::registry::approve_wasm(316, 1);

    let aaa = Principal::from_slice(&[216, 1]);
    let owner = Principal::from_slice(&[216, 2]);
    pre_register_aaa(
        &RegisterArgs {
            canister_id: aaa,
            owner,
            name: "S16-Tier-Gate".into(),
            avatar_seed: 1,
        },
        1_000,
    )
    .unwrap();

    apply_event(&Event {
        v: 1,
        id: 1,
        at: 1,
        aaa,
        owner,
        kind: EventKind::Classified {
            classification_id: 1,
            subject_id: 1,
            gold: Some((2, 2)),
            fee: 0,
        },
    });
    assert!(get_progress(&aaa).tier < 2);
    assert!(get_leaderboard(None, 200)
        .items
        .iter()
        .all(|row| row.aaa != aaa));

    for i in 2..=60u64 {
        apply_event(&Event {
            v: 1,
            id: i,
            at: i,
            aaa,
            owner,
            kind: EventKind::Classified {
                classification_id: i,
                subject_id: i as u32,
                gold: Some((2, 2)),
                fee: 0,
            },
        });
    }

    assert!(get_progress(&aaa).tier >= 2);
    assert!(get_leaderboard(None, 200)
        .items
        .iter()
        .any(|row| row.aaa == aaa));
}
