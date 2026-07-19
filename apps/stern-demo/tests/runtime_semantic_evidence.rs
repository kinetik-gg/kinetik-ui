//! Focused generator and stale-evidence rejection checks.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

#[test]
fn generator_records_honest_current_runtime_packet() {
    let evidence = temp("generated");
    generate(&evidence);
    let record: Value = serde_json::from_slice(&fs::read(&evidence).expect("evidence bytes"))
        .expect("evidence JSON");

    assert_eq!(record["status"], "incomplete");
    assert_eq!(
        record["runtime"]["components"].as_array().unwrap().len(),
        34
    );
    assert_eq!(passed(&record["runtime"]["components"]), 31);
    assert_eq!(record["runtime"]["journeys"].as_array().unwrap().len(), 7);
    assert_eq!(passed(&record["runtime"]["journeys"]), 5);
    assert_eq!(record["semanticSnapshots"].as_array().unwrap().len(), 2);
    assert_eq!(record["publicConsumerAudit"]["passed"], true);
    assert_eq!(
        gate(&record, "renderer-and-scale-quality")["status"],
        "pending"
    );
    assert_eq!(gate(&record, "platform-integration")["status"], "pending");
    verify(&evidence, true);

    let _ = fs::remove_file(evidence);
}

#[test]
fn verifier_rejects_stale_source_and_premature_gate_claims() {
    let evidence = temp("tampered");
    generate(&evidence);
    let mut record: Value = serde_json::from_slice(&fs::read(&evidence).expect("evidence bytes"))
        .expect("evidence JSON");

    record["source"]["tree"] = Value::String("0".repeat(40));
    fs::write(&evidence, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    verify(&evidence, false);

    generate(&evidence);
    let mut record: Value = serde_json::from_slice(&fs::read(&evidence).expect("evidence bytes"))
        .expect("evidence JSON");
    gate_mut(&mut record, "renderer-and-scale-quality")["status"] = Value::String("passed".into());
    fs::write(&evidence, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    verify(&evidence, false);

    let _ = fs::remove_file(evidence);
}

fn generate(path: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_runtime_semantic_evidence"))
        .args(["--output", path.to_str().unwrap(), "--source-ref", "HEAD"])
        .current_dir(repo_root())
        .status()
        .expect("run evidence generator");
    assert!(status.success());
}

fn verify(path: &Path, expected: bool) {
    let output = Command::new("node")
        .args([
            "apps/stern-demo/tools/check-runtime-semantic-evidence.mjs",
            "--evidence",
            path.to_str().unwrap(),
            "--source-ref",
            "HEAD",
        ])
        .current_dir(repo_root())
        .output()
        .expect("run evidence verifier");
    assert_eq!(
        output.status.success(),
        expected,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn passed(records: &Value) -> usize {
    records
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["status"] == "passed")
        .count()
}

fn gate<'a>(record: &'a Value, id: &str) -> &'a Value {
    record["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|gate| gate["id"] == id)
        .expect("gate")
}

fn gate_mut<'a>(record: &'a mut Value, id: &str) -> &'a mut Value {
    record["gates"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|gate| gate["id"] == id)
        .expect("gate")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "stern-runtime-semantic-evidence-{label}-{}.json",
        std::process::id()
    ))
}
