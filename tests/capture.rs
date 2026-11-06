use cobol_to_rust::capture::{Capture, ProcessStatus, capture, compare, snapshot};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-work/capture")
        .join(format!("{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[test]
fn child_emits_fixture() {
    let Some(mode) = std::env::var_os("C2R_CAPTURE_CHILD") else {
        return;
    };
    match mode.to_str().unwrap() {
        "binary" => {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(b"out\0\xff\n").unwrap();
            stdout.flush().unwrap();
            let mut stderr = std::io::stderr().lock();
            stderr.write_all(b"err\r\n").unwrap();
            stderr.flush().unwrap();
            let root = std::env::var_os("C2R_CAPTURE_ROOT").unwrap();
            fs::write(PathBuf::from(root).join("produced.bin"), [0, 255, 10]).unwrap();
            std::process::exit(0);
        }
        "flood" => {
            std::io::stdout()
                .write_all(&vec![0x41; 1024 * 1024])
                .unwrap();
            std::io::stderr()
                .write_all(&vec![0xff; 1024 * 1024])
                .unwrap();
            std::process::exit(0);
        }
        "timeout" => std::thread::sleep(Duration::from_secs(30)),
        "exit7" => std::process::exit(7),
        _ => panic!("unknown child mode"),
    }
}

#[test]
fn captures_direct_process_bytes_and_before_after_tree() {
    let dir = temp_dir("capture");
    fs::write(dir.join("input.dat"), [1, 2, 0]).unwrap();
    let mut env = BTreeMap::new();
    env.insert(
        OsString::from("C2R_CAPTURE_CHILD"),
        OsString::from("binary"),
    );
    env.insert(
        OsString::from("C2R_CAPTURE_ROOT"),
        dir.as_os_str().to_owned(),
    );
    let captured = capture(
        &std::env::current_exe().unwrap(),
        &[
            "--exact".into(),
            "child_emits_fixture".into(),
            "--nocapture".into(),
        ],
        &env,
        &dir,
        &dir,
        Duration::from_secs(15),
    )
    .unwrap();
    assert!(captured.status.success);
    assert!(captured.stdout.starts_with(b"\nrunning 1 test\n"));
    assert!(captured.stdout.ends_with(b"out\0\xff\n"));
    assert_eq!(
        captured.stdout.len(),
        b"\nrunning 1 test\nout\0\xff\n".len()
    );
    assert_eq!(captured.stderr, b"err\r\n");
    assert_eq!(captured.before["input.dat"], [1, 2, 0]);
    assert_eq!(captured.after["produced.bin"], [0, 255, 10]);
    assert_eq!(captured.after["input.dat"], [1, 2, 0]);
    fs::remove_dir_all(dir).unwrap();
}

fn capture_child(
    dir: &std::path::Path,
    mode: &str,
    timeout: Duration,
) -> Result<cobol_to_rust::capture::Capture, cobol_to_rust::capture::CaptureError> {
    let mut env = BTreeMap::new();
    env.insert(OsString::from("C2R_CAPTURE_CHILD"), OsString::from(mode));
    env.insert(
        OsString::from("C2R_CAPTURE_ROOT"),
        dir.as_os_str().to_owned(),
    );
    capture(
        &std::env::current_exe().unwrap(),
        &[
            "--exact".into(),
            "child_emits_fixture".into(),
            "--nocapture".into(),
        ],
        &env,
        dir,
        dir,
        timeout,
    )
}

#[test]
fn drains_large_stdout_and_stderr_concurrently() {
    let dir = temp_dir("flood");
    let captured = capture_child(&dir, "flood", Duration::from_secs(15)).unwrap();
    assert!(captured.status.success);
    let mut expected_stdout = b"\nrunning 1 test\n".to_vec();
    expected_stdout.extend(vec![0x41; 1024 * 1024]);
    assert_eq!(captured.stdout, expected_stdout);
    assert_eq!(captured.stderr, vec![0xff; 1024 * 1024]);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn returns_child_exit_status_and_timeout_as_distinct_outcomes() {
    let dir = temp_dir("status");
    let exited = capture_child(&dir, "exit7", Duration::from_secs(15)).unwrap();
    assert_eq!(exited.status.code, Some(7));
    assert!(!exited.status.success);
    let timed_out = capture_child(&dir, "timeout", Duration::from_millis(100));
    assert!(matches!(
        timed_out,
        Err(cobol_to_rust::capture::CaptureError::Timeout(_))
    ));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn snapshots_nested_empty_and_binary_files_in_sorted_relative_names() {
    let dir = temp_dir("snapshot");
    fs::create_dir(dir.join("nested")).unwrap();
    fs::write(dir.join("z"), []).unwrap();
    fs::write(dir.join("nested").join("a.bin"), [0, 255, 1]).unwrap();
    let tree = snapshot(&dir).unwrap();
    assert_eq!(
        tree.keys().map(String::as_str).collect::<Vec<_>>(),
        ["nested/a.bin", "z"]
    );
    assert_eq!(tree["z"], Vec::<u8>::new());
    assert_eq!(tree["nested/a.bin"], [0, 255, 1]);
    fs::remove_dir_all(dir).unwrap();
}

fn sample() -> Capture {
    Capture {
        status: ProcessStatus {
            code: Some(0),
            success: true,
        },
        stdout: b"same\n".to_vec(),
        stderr: b"".to_vec(),
        before: BTreeMap::from([("input".into(), vec![0, 1])]),
        after: BTreeMap::from([
            ("input".into(), vec![0, 1]),
            ("empty".into(), vec![]),
            ("out".into(), vec![1, 2, 3]),
        ]),
    }
}

type Mutation = Box<dyn Fn(&mut Capture)>;

#[test]
fn comparator_requires_every_byte_and_status_to_match() {
    let baseline = sample();
    assert!(compare(&baseline, &baseline).equal);
    let mut actual = baseline.clone();
    actual.stdout = b"same\n ".to_vec();
    let stdout = assert_single_difference(&baseline, &actual, "stdout");
    assert_eq!(stdout.first_diff_offset, Some(5));
    assert_eq!(stdout.expected_len, Some(5));
    assert_eq!(stdout.actual_len, Some(6));
    assert_eq!(
        stdout.expected_context_hex.as_deref(),
        Some("73 61 6d 65 0a")
    );
    assert_eq!(
        stdout.actual_context_hex.as_deref(),
        Some("73 61 6d 65 0a 20")
    );
    assert!(
        stdout
            .expected_context_hex
            .as_ref()
            .unwrap()
            .split_whitespace()
            .count()
            <= 16
    );
}

fn assert_single_difference(
    expected: &Capture,
    actual: &Capture,
    artifact: &str,
) -> cobol_to_rust::capture::Difference {
    let result = compare(expected, actual);
    assert!(!result.equal);
    assert_eq!(result.differences.len(), 1);
    assert_eq!(result.differences[0].artifact, artifact);
    result.differences.into_iter().next().unwrap()
}

#[test]
fn comparator_reports_each_independent_parity_defect() {
    let expected = sample();
    let cases: Vec<(&str, Mutation)> = vec![
        ("stdout", Box::new(|c| c.stdout = b"same ".to_vec())),
        ("stdout", Box::new(|c| c.stdout = b"same".to_vec())),
        ("stdout", Box::new(|c| c.stdout = b"same\r\n".to_vec())),
        ("stdout", Box::new(|c| c.stdout = b"same\nsame\n".to_vec())),
        ("stdout", Box::new(|c| c.stdout = b"same\0\xff\n".to_vec())),
        ("stderr", Box::new(|c| c.stderr = b"diagnostic\n".to_vec())),
        ("process status", Box::new(|c| c.status.code = Some(1))),
        (
            "before/input",
            Box::new(|c| c.before.get_mut("input").unwrap()[0] ^= 1),
        ),
        (
            "before/input",
            Box::new(|c| {
                c.before.remove("input");
            }),
        ),
        (
            "after/extra",
            Box::new(|c| {
                c.after.insert("extra".into(), vec![1]);
            }),
        ),
        (
            "after/extra-empty",
            Box::new(|c| {
                c.after.insert("extra-empty".into(), vec![]);
            }),
        ),
        (
            "after/empty",
            Box::new(|c| {
                c.after.remove("empty");
            }),
        ),
        (
            "after/out",
            Box::new(|c| {
                c.after.remove("out");
            }),
        ),
        (
            "after/input",
            Box::new(|c| {
                c.after.get_mut("input").unwrap()[0] ^= 1;
            }),
        ),
        (
            "after/input",
            Box::new(|c| {
                c.after.remove("input");
            }),
        ),
        (
            "after/empty",
            Box::new(|c| c.after.get_mut("empty").unwrap().push(0)),
        ),
        (
            "after/out",
            Box::new(|c| c.after.get_mut("out").unwrap()[1] ^= 1),
        ),
    ];
    for (artifact, mutate) in cases {
        let mut actual = expected.clone();
        mutate(&mut actual);
        assert_single_difference(&expected, &actual, artifact);
    }
    let mut reordered = expected.clone();
    reordered.stdout = b"two\none\n".to_vec();
    let mut ordered = expected;
    ordered.stdout = b"one\ntwo\n".to_vec();
    assert_single_difference(&ordered, &reordered, "stdout");
}

#[test]
fn comparator_detects_record_reordering_and_duplicates() {
    let mut expected = sample();
    expected.stdout = b"one\ntwo\nrepeat\nrepeat\n".to_vec();
    let mut actual = expected.clone();
    actual.stdout = b"two\none\nrepeat\n".to_vec();
    let result = compare(&expected, &actual);
    assert!(!result.equal);
    assert_eq!(result.differences[0].artifact, "stdout");
    assert_eq!(result.differences[0].first_diff_offset, Some(0));
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_symlinks() {
    use std::os::unix::fs::symlink;
    let dir = temp_dir("symlink");
    let target = dir.join("target");
    fs::write(&target, b"x").unwrap();
    symlink(&target, dir.join("link")).unwrap();
    assert!(snapshot(&dir).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_non_utf8_names() {
    use std::os::unix::ffi::OsStringExt;
    let dir = temp_dir("nonutf8");
    fs::write(dir.join(OsString::from_vec(vec![b'f', 0xff])), b"x").unwrap();
    assert!(snapshot(&dir).is_err());
    fs::remove_dir_all(dir).unwrap();
}
