use std::time::Duration;

use candid::encode_one;
use integration_tests::step;
use integration_tests::world::World;
use sc_types::{ApiError, ClassificationReceipt};

#[test]
fn t3_6_submit_retried_after_sys_unknown_produces_one_classification_and_one_record() {
    println!("T3.6 demo: 03 §8.3 — a submit whose reply is lost (bounded-wait timeout → SYS_UNKNOWN) is retried once and yields exactly one platform classification and one local record");
    let w = World::new("SysUnknownSurveyor-01", 1);
    let task = w.get_task();
    let sub = w.submission(&task);
    step("operator fetched a task via the AAA relay");

    let msg = w
        .env
        .pic
        .submit_call(
            w.aaa,
            w.operator,
            "submit_classification",
            encode_one(&sub).unwrap(),
        )
        .unwrap();
    w.env.pic.tick();
    w.env.pic.advance_time(Duration::from_secs(120));
    step("the AAA forwarded the submit; IC time jumped past its 60 s bounded-wait deadline while the platform was mid-call");

    let res: Result<ClassificationReceipt, ApiError> = w.await_ingress(msg);
    let receipt = res.expect("submit_classification must succeed via the retry");
    assert!(
        receipt.duplicate,
        "the first attempt landed but its reply was lost, so the retry must get the original receipt"
    );
    step("the deadline expired as SYS_UNKNOWN; the AAA retried once and got the original receipt (duplicate = true)");

    let rec = |seq: u64| {
        let r: Result<Option<aaa::record::Record>, ApiError> =
            w.env.query(w.aaa, w.owner, "get_record", seq);
        r.unwrap()
    };
    assert!(rec(1).is_some(), "exactly one local record must exist");
    assert!(
        rec(2).is_none(),
        "the retry must not create a second local record"
    );
    let class = |id: u64| {
        let c: Option<platform::scoring::Classification> =
            w.env.query(w.platform, w.admin, "get_classification", id);
        c
    };
    assert!(class(1).is_some(), "one platform classification");
    assert!(
        class(2).is_none(),
        "the retry must not produce a second platform classification"
    );
    step("exactly ONE platform classification and ONE local record (acceptance criterion)");
}
