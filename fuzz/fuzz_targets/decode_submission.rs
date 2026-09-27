#![no_main]
use libfuzzer_sys::fuzz_target;
use sc_types::{ClassificationSubmission, ReviewSubmission, Task};

fuzz_target!(|data: &[u8]| {
    let _ = candid::decode_one::<ClassificationSubmission>(data);
    let _ = candid::decode_one::<ReviewSubmission>(data);
    let _ = candid::decode_one::<Task>(data);
});
