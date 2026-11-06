use cobol_to_rust::workbench::safe_relative;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_root() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-work/workbench")
        .join(format!("{}-{nonce}", std::process::id()));
    fs::create_dir_all(path.join("cases")).unwrap();
    path.canonicalize().unwrap()
}

#[test]
fn safe_relative_accepts_nested_names_and_rejects_traversal_or_roots() {
    assert_eq!(
        safe_relative("folder/input.dat").unwrap().to_str(),
        Some("folder/input.dat")
    );
    for unsafe_name in [
        "",
        "../outside",
        "a/../../outside",
        "/absolute",
        "C:\\outside",
        "folder\\file",
        "a:b",
    ] {
        assert!(
            safe_relative(unsafe_name).is_err(),
            "accepted {unsafe_name:?}"
        );
    }
}

#[test]
fn verify_all_fails_for_blocked_and_missing_required_cases_without_profile() {
    let root = fixture_root();
    fs::write(
        root.join("cases/required.json"),
        br#"["blocked","missing"]"#,
    )
    .unwrap();
    fs::write(
        root.join("cases/blocked.json"),
        br#"{"schema_version":1,"id":"blocked","admission":"blocked","blockers":["not admitted"],"reviewed":false,"source_revision":"","sources":[],"build_args":[],"reference_args":[],"repetitions":0,"timeout_ms":0,"semantics":"","scenarios":[]}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_c2r"))
        .args(["verify", "--all"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"status\":\"blocked\""));
    assert!(String::from_utf8_lossy(&output.stderr).contains("infrastructure_error"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn assess_rejects_manifest_with_missing_required_fields() {
    let root = fixture_root();
    fs::write(
        root.join("cases/bad.json"),
        br#"{"schema_version":1,"id":"bad"}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_c2r"))
        .args(["assess", "bad"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("infrastructure_error"));
    fs::remove_dir_all(root).unwrap();
}
