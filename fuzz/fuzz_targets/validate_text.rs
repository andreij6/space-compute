#![no_main]
use libfuzzer_sys::fuzz_target;
use sc_types::limits;

fuzz_target!(|text: &str| {
    let _ = limits::aaa_name(text);
    let _ = limits::rationale(text);
    let _ = limits::operator_label(text);
    let _ = limits::agent_label(&Some(text.to_string()));
    let _ = limits::field(text);
});
