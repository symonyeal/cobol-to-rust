use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capture {
    pub status: ProcessStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub before: BTreeMap<String, Vec<u8>>,
    pub after: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessStatus {
    pub code: Option<i32>,
    pub success: bool,
}

impl From<ExitStatus> for ProcessStatus {
    fn from(status: ExitStatus) -> Self {
        Self {
            code: status.code(),
            success: status.success(),
        }
    }
}

#[derive(Debug)]
pub enum CaptureError {
    Io(io::Error),
    UnsafePath(PathBuf),
    NonUtf8Path(PathBuf),
    Timeout(Duration),
    ReaderPanicked,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "capture I/O failed: {e}"),
            Self::UnsafePath(p) => {
                write!(f, "unsafe or non-regular snapshot path: {}", p.display())
            }
            Self::NonUtf8Path(p) => write!(f, "snapshot path is not UTF-8: {}", p.display()),
            Self::Timeout(d) => write!(f, "process exceeded timeout of {} ms", d.as_millis()),
            Self::ReaderPanicked => f.write_str("stdout or stderr reader panicked"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<io::Error> for CaptureError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Runs a program directly and records its raw output and the watched tree on both sides.
/// INPUT: `program`, `args`, `env`, `cwd`, `snapshot_dir`, and a positive `timeout`.
/// OUTPUT: exact process bytes/status and UTF-8 relative-path file contents; timeout and
/// filesystem/process failures are errors, never parity results.
pub fn capture(
    program: &Path,
    args: &[OsString],
    env: &BTreeMap<OsString, OsString>,
    cwd: &Path,
    snapshot_dir: &Path,
    timeout: Duration,
) -> Result<Capture, CaptureError> {
    let before = snapshot(snapshot_dir)?;
    let mut command = Command::new(program);
    command
        .args(args)
        .env_clear()
        .envs(env)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("stdout pipe unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("stderr pipe unavailable"))?;
    let out_reader = thread::spawn(move || read_all(stdout));
    let err_reader = thread::spawn(move || read_all(stderr));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            child.kill()?;
            let _ = child.wait()?;
            let _ = out_reader.join();
            let _ = err_reader.join();
            return Err(CaptureError::Timeout(timeout));
        }
        thread::sleep(Duration::from_millis(5));
    };
    let stdout = out_reader
        .join()
        .map_err(|_| CaptureError::ReaderPanicked)??;
    let stderr = err_reader
        .join()
        .map_err(|_| CaptureError::ReaderPanicked)??;
    let after = snapshot(snapshot_dir)?;
    Ok(Capture {
        status: status.into(),
        stdout,
        stderr,
        before,
        after,
    })
}

fn read_all(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Returns all regular files under `root` keyed by sorted, safe, UTF-8 relative names.
pub fn snapshot(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, CaptureError> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(CaptureError::UnsafePath(root.to_path_buf()));
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files)?;
    Ok(files)
}

fn walk(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CaptureError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(CaptureError::UnsafePath(path));
        }
        if metadata.is_dir() {
            walk(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| CaptureError::UnsafePath(path.clone()))?;
            let name = safe_name(relative)?;
            files.insert(name, fs::read(path)?);
        } else {
            return Err(CaptureError::UnsafePath(path));
        }
    }
    Ok(())
}

fn safe_name(path: &Path) -> Result<String, CaptureError> {
    let safe = path.components().all(|c| {
        matches!(c, Component::Normal(s)
        if !s.to_string_lossy().chars().any(|ch| matches!(ch, '/' | '\\' | ':')))
    });
    if !safe {
        return Err(CaptureError::UnsafePath(path.to_path_buf()));
    }
    let name = path
        .to_str()
        .ok_or_else(|| CaptureError::NonUtf8Path(path.to_path_buf()))?;
    Ok(name.replace('\\', "/"))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Difference {
    pub artifact: String,
    pub expected_len: Option<usize>,
    pub actual_len: Option<usize>,
    pub first_diff_offset: Option<usize>,
    pub expected_context_hex: Option<String>,
    pub actual_context_hex: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Comparison {
    pub equal: bool,
    pub differences: Vec<Difference>,
}

/// Compares every acceptance byte and reports a bounded hex window around the first difference.
pub fn compare(expected: &Capture, actual: &Capture) -> Comparison {
    let mut differences = Vec::new();
    if expected.status != actual.status {
        differences.push(Difference {
            artifact: "process status".into(),
            expected_len: None,
            actual_len: None,
            first_diff_offset: None,
            expected_context_hex: None,
            actual_context_hex: None,
        });
    }
    compare_bytes(
        "stdout",
        Some(&expected.stdout),
        Some(&actual.stdout),
        &mut differences,
    );
    compare_bytes(
        "stderr",
        Some(&expected.stderr),
        Some(&actual.stderr),
        &mut differences,
    );
    compare_trees("before", &expected.before, &actual.before, &mut differences);
    compare_trees("after", &expected.after, &actual.after, &mut differences);
    Comparison {
        equal: differences.is_empty(),
        differences,
    }
}

fn compare_trees(
    label: &str,
    expected: &BTreeMap<String, Vec<u8>>,
    actual: &BTreeMap<String, Vec<u8>>,
    out: &mut Vec<Difference>,
) {
    let names: BTreeSet<_> = expected.keys().chain(actual.keys()).collect();
    for name in names {
        compare_bytes(
            &format!("{label}/{name}"),
            expected.get(name),
            actual.get(name),
            out,
        );
    }
}

fn compare_bytes(
    label: &str,
    expected: Option<&Vec<u8>>,
    actual: Option<&Vec<u8>>,
    out: &mut Vec<Difference>,
) {
    if expected == actual {
        return;
    }
    let (offset, expected_context_hex, actual_context_hex) = match (expected, actual) {
        (Some(a), Some(b)) => {
            let offset = a
                .iter()
                .zip(b)
                .position(|(x, y)| x != y)
                .unwrap_or(a.len().min(b.len()));
            (
                Some(offset),
                Some(context(a, offset)),
                Some(context(b, offset)),
            )
        }
        _ => (
            None,
            expected.map(|b| context(b, 0)),
            actual.map(|b| context(b, 0)),
        ),
    };
    out.push(Difference {
        artifact: label.into(),
        expected_len: expected.map(Vec::len),
        actual_len: actual.map(Vec::len),
        first_diff_offset: offset,
        expected_context_hex,
        actual_context_hex,
    });
}

fn context(bytes: &[u8], offset: usize) -> String {
    let start = offset.saturating_sub(8);
    let end = bytes.len().min(offset.saturating_add(8));
    bytes[start..end]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
