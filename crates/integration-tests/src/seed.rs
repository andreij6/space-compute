use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use candid::{encode_args, encode_one};
use flate2::write::GzEncoder;
use flate2::Compression;
use platform::catalog::SubjectInput;
use sc_types::{Answer, Protocol, SubjectRef};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const BATCH: usize = 500;

#[derive(Deserialize)]
pub struct ManifestRow {
    pub subject_id: u32,
    pub field: String,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub rgb_url: String,
    pub rgb_sha256: String,
    pub dossier_url: String,
    pub dossier_sha256: String,
    pub data_version: u16,
}

#[derive(Deserialize)]
struct GoldRow {
    answers: BTreeMap<String, String>,
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex sha256"))
        .collect()
}

pub fn protocol(path: &Path) -> Protocol {
    serde_json::from_str(&std::fs::read_to_string(path).expect("protocol json")).expect("protocol")
}

pub fn subjects(manifest: &Path, gold: &Path, base_url: &str, limit: usize) -> Vec<SubjectInput> {
    let gold: BTreeMap<String, GoldRow> =
        serde_json::from_str(&std::fs::read_to_string(gold).expect("gold json")).expect("gold");
    let base = base_url.trim_end_matches('/');
    let text = std::fs::read_to_string(manifest).expect("manifest");
    let rows: Vec<&str> = text.lines().collect();
    let stride = (rows.len() / limit.max(1)).max(1);
    rows.iter()
        .step_by(stride)
        .take(limit)
        .map(|l| {
            let r: ManifestRow = serde_json::from_str(l).expect("manifest row");
            let answers = gold.get(&r.subject_id.to_string()).map(|g| {
                g.answers
                    .iter()
                    .map(|(q, a)| Answer {
                        question_id: q.clone(),
                        answer_id: a.clone(),
                    })
                    .collect()
            });
            SubjectInput {
                subject: SubjectRef {
                    subject_id: r.subject_id,
                    field: r.field,
                    ra_deg: r.ra_deg,
                    dec_deg: r.dec_deg,
                    image_url: format!("{base}/{}", r.rgb_url),
                    image_sha256: hex(&r.rgb_sha256),
                    dossier_url: format!("{base}/{}", r.dossier_url),
                    dossier_sha256: hex(&r.dossier_sha256),
                    data_version: r.data_version,
                },
                gold: answers,
            }
        })
        .collect()
}

pub fn wasm_upload(version: u32, module: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut gz = GzEncoder::new(Vec::new(), Compression::best());
    gz.write_all(module).expect("gzip");
    let blob = gz.finish().expect("gzip");
    let sha = Sha256::digest(&blob).to_vec();
    let arg = encode_args((version, blob.clone(), sha)).expect("encode");
    (blob, arg)
}

pub fn write_args(
    out: &Path,
    protocol: &Protocol,
    subjects: &[SubjectInput],
    aaa_wasm: &[u8],
) -> usize {
    std::fs::create_dir_all(out).expect("out dir");
    std::fs::write(
        out.join("protocol.bin"),
        encode_one(protocol).expect("encode"),
    )
    .expect("write");
    for old in std::fs::read_dir(out).expect("out dir").flatten() {
        if old.file_name().to_string_lossy().starts_with("subjects_") {
            std::fs::remove_file(old.path()).expect("clean");
        }
    }
    let batches = subjects.chunks(BATCH).collect::<Vec<_>>();
    for (i, b) in batches.iter().enumerate() {
        std::fs::write(
            out.join(format!("subjects_{i:03}.bin")),
            encode_one(b.to_vec()).expect("encode"),
        )
        .expect("write");
    }
    std::fs::write(out.join("aaa_wasm.bin"), wasm_upload(1, aaa_wasm).1).expect("write");
    batches.len()
}
