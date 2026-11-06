//! Baseline capture and verification are separate operations. Expected observations are never rewritten by verification.
use crate::{
    Result,
    capture::{self, Capture},
    digest,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    time::Duration,
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub root: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub domain: String,
    pub initial_hex: BTreeMap<String, String>,
    pub port_args: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub schema_version: u32,
    pub id: String,
    pub admission: String,
    pub blockers: Vec<String>,
    pub reviewed: bool,
    pub source_revision: String,
    pub sources: Vec<Source>,
    pub build_args: Vec<String>,
    pub reference_args: Vec<String>,
    pub repetitions: usize,
    pub timeout_ms: u64,
    pub semantics: String,
    pub scenarios: Vec<Scenario>,
}

/// A local execution profile. Every compiler/runtime file in `files` is rehashed before use.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub id: String,
    pub platform: String,
    pub reference_architecture: String,
    pub root: String,
    pub cobc: String,
    pub files: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
    pub compiler_info_sha256: String,
}

#[derive(Deserialize, Serialize)]
struct Baseline {
    schema_version: u32,
    case: String,
    manifest_sha256: String,
    profile_sha256: String,
    job_path_sha256: String,
    compiler_info: Vec<u8>,
    reference_binary_sha256: String,
    captures: BTreeMap<String, Vec<Capture>>,
}

pub fn safe_relative(value: &str) -> Result<&Path> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains(['\\', ':'])
        || !path.components().all(|p| matches!(p, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path: {value}").into());
    }
    Ok(path)
}

fn identifier(value: &str) -> Result<&str> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("identifier must contain only ASCII letters, digits or hyphens".into());
    }
    Ok(value)
}

fn canonical(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = fs::canonicalize(path)?;
    // GCC's Windows argument parser does not accept Rust's extended-length prefix.
    #[cfg(windows)]
    if let Some(value) = path.to_str().and_then(|s| s.strip_prefix("\\\\?\\")) {
        return Ok(PathBuf::from(
            if let Some(unc) = value.strip_prefix("UNC\\") {
                format!("\\\\{unc}")
            } else {
                value.to_string()
            },
        ));
    }
    Ok(path)
}

fn within(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = canonical(root.join(safe_relative(relative)?))?;
    if !path.starts_with(root) {
        return Err(format!("path escapes declared root: {relative}").into());
    }
    Ok(path)
}

fn profile_paths(
    root: &Path,
    dir: &Path,
    paths: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !canonical(&path)?.starts_with(root) {
            return Err("profile contains an unsafe link".into());
        }
        if metadata.is_dir() {
            profile_paths(root, &path, paths)?;
        } else if metadata.is_file() {
            paths.insert(
                path.strip_prefix(root)?
                    .to_str()
                    .ok_or("profile filename is not UTF-8")?
                    .replace('\\', "/"),
            );
        } else {
            return Err("profile contains a non-regular file".into());
        }
    }
    Ok(())
}

fn load(id: &str) -> Result<(Case, Vec<u8>)> {
    identifier(id)?;
    let bytes = fs::read(format!("cases/{id}.json"))?;
    let case: Case = serde_json::from_slice(&bytes)?;
    if case.schema_version != 1 || case.id != id {
        return Err("unsupported or inconsistent case manifest".into());
    }
    Ok((case, bytes))
}

pub fn assess(id: &str) -> Result<bool> {
    let (case, _) = load(id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"schema_version":1,"case":case.id,"admission":case.admission,"reviewed":case.reviewed,"source_revision":case.source_revision,"source_files":case.sources,"blockers":case.blockers,"semantic_risks":case.semantics,"build_requirements":{"reference":"pinned native Windows GnuCOBOL 3.2 and compatible C compiler/runtime","arguments":case.build_args,"rust":"rust-toolchain.toml and Cargo.lock","input_fixtures":"initial_hex in each scenario","file_observations":"all regular files before and after job"},"scenario_count":case.scenarios.len()})
        )?
    );
    Ok(case.admission == "admitted")
}

fn unhex(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return Err("invalid hexadecimal fixture".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(Into::into))
        .collect()
}

fn initial(scenario: &Scenario) -> Result<BTreeMap<String, Vec<u8>>> {
    scenario
        .initial_hex
        .iter()
        .map(|(p, hex)| {
            safe_relative(p)?;
            Ok((p.clone(), unhex(hex)?))
        })
        .collect()
}

fn restore(job: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    // `job` is exclusively owned by this invocation's lock, beneath a validated run root.
    if job.exists() {
        if canonical(job)? != job {
            return Err("job path is redirected through a filesystem link".into());
        }
        capture::snapshot(job)?; // Reject symlinks before removing regenerable run state.
        fs::remove_dir_all(job)?;
    }
    fs::create_dir_all(job)?;
    for (name, bytes) in files {
        let path = job.join(safe_relative(name)?);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
    }
    Ok(())
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn flag<'a>(args: &'a [String], key: &str) -> Result<&'a str> {
    let i = args
        .iter()
        .position(|v| v == key)
        .ok_or_else(|| format!("missing {key}"))?;
    args.get(i + 1)
        .map(String::as_str)
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| format!("missing value for {key}").into())
}

pub fn execute(args: &[String]) -> Result<bool> {
    let operation = args.first().ok_or("missing operation")?;
    if operation == "verify" && args.get(1).map(String::as_str) == Some("--all") {
        let ids: Vec<String> = serde_json::from_slice(&fs::read("cases/required.json")?)?;
        if ids.is_empty()
            || ids.iter().collect::<std::collections::BTreeSet<_>>().len() != ids.len()
        {
            return Err("required case registry is empty".into());
        }
        let mut all = true;
        let mut errors = Vec::new();
        for id in ids {
            let mut argv = args.to_vec();
            argv[1] = id;
            match execute(&argv) {
                Ok(pass) => all &= pass,
                Err(e) => {
                    all = false;
                    errors.push(e.to_string());
                }
            }
        }
        if !errors.is_empty() {
            return Err(
                format!("required case infrastructure errors: {}", errors.join("; ")).into(),
            );
        }
        return Ok(all);
    }
    let (case, manifest) = load(args.get(1).ok_or("missing case")?)?;
    if case.admission != "admitted"
        || !case.blockers.is_empty()
        || (operation == "verify" && !case.reviewed)
    {
        println!(
            "{}",
            serde_json::json!({"schema_version":1,"case":case.id,"status":"blocked","blockers":case.blockers})
        );
        return Ok(false);
    }
    if case.repetitions < 3
        || case.scenarios.is_empty()
        || case.timeout_ms == 0
        || case.sources.is_empty()
    {
        return Err("incomplete baseline contract".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    for scenario in &case.scenarios {
        identifier(&scenario.id)?;
        if !ids.insert(&scenario.id) {
            return Err("duplicate scenario id".into());
        }
        initial(scenario)?;
    }
    let profile_bytes = fs::read(flag(args, "--profile")?)?;
    let profile: Profile = serde_json::from_slice(&profile_bytes)?;
    if profile.schema_version != 1
        || profile.platform != format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    {
        return Err("unsupported execution profile platform".into());
    }
    let root = canonical(&profile.root)?;
    if profile.files.is_empty() || !profile.files.contains_key(&profile.cobc) {
        return Err("profile does not pin its compiler".into());
    }
    let mut paths = std::collections::BTreeSet::new();
    profile_paths(&root, &root, &mut paths)?;
    if paths != profile.files.keys().cloned().collect() {
        return Err("profile file set changed (missing or unpinned artifact)".into());
    }
    for (name, expected) in &profile.files {
        let path = within(&root, name)?;
        if fs::symlink_metadata(&path)?.file_type().is_symlink()
            || digest(&fs::read(&path)?) != *expected
        {
            return Err(format!("profile artifact changed: {name}").into());
        }
    }
    let mut env: BTreeMap<OsString, OsString> = BTreeMap::new();
    for (key, value) in &profile.environment {
        env.insert(
            key.into(),
            value.replace("${ROOT}", &root.to_string_lossy()).into(),
        );
    }
    let external = canonical(flag(args, "--sources")?)?;
    let repo = canonical(".")?;
    let mut sources = Vec::new();
    for source in &case.sources {
        let base = match source.root.as_str() {
            "external" => &external,
            "repository" => &repo,
            _ => return Err("unknown source root".into()),
        };
        let path = within(base, &source.path)?;
        if digest(&fs::read(&path)?) != source.sha256 {
            return Err(format!("source changed: {}", source.path).into());
        }
        sources.push(path.into_os_string());
    }
    let runs = PathBuf::from(flag(args, "--runs")?);
    fs::create_dir_all(&runs)?;
    let runs = canonical(runs)?;
    let case_dir = runs.join(&case.id);
    fs::create_dir_all(&case_dir)?;
    if canonical(&case_dir)? != case_dir {
        return Err("case directory is redirected through a filesystem link".into());
    }
    let lock_path = case_dir.join("job.lock");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)?
        .write_all(b"c2r exclusive job owner\n")?;
    let _lock = Lock(lock_path);
    let job = case_dir.join("job");
    let label = identifier(flag(args, "--label")?)?;
    let baseline_path = case_dir.join(format!("baseline-{label}.json"));
    let profile_sha256 = digest(&profile_bytes);
    let job_path_sha256 = digest(job.as_os_str().to_string_lossy().as_bytes());
    let manifest_sha256 = digest(&manifest);
    let timeout = Duration::from_millis(case.timeout_ms);
    if operation == "baseline" {
        if baseline_path.exists() {
            return Err("baseline exists; use a new label to retain both captures".into());
        }
        let build = case_dir.join(format!("build-{label}"));
        fs::create_dir(&build)?;
        let info = capture::capture(
            &root.join(&profile.cobc),
            &["--info".into()],
            &env,
            &build,
            &build,
            timeout,
        )?;
        fs::write(
            build.join("compiler-info.json"),
            serde_json::to_vec_pretty(&info)?,
        )?;
        if !info.status.success
            || !info.stderr.is_empty()
            || digest(&info.stdout) != profile.compiler_info_sha256
        {
            return Err("compiler identity check failed".into());
        }
        let exe = build.join(if cfg!(windows) {
            "reference.exe"
        } else {
            "reference"
        });
        let mut build_args: Vec<OsString> = case.build_args.iter().map(Into::into).collect();
        build_args.extend([OsString::from("-o"), exe.as_os_str().to_owned()]);
        build_args.extend(sources);
        let built = capture::capture(
            &root.join(&profile.cobc),
            &build_args,
            &env,
            &build,
            &build,
            timeout,
        )?;
        fs::write(build.join("build.json"), serde_json::to_vec_pretty(&built)?)?;
        if !built.status.success {
            return Err("COBOL build failed (retained build.json)".into());
        }
        let mut captures = BTreeMap::new();
        for scenario in &case.scenarios {
            let mut repetitions = Vec::new();
            for _ in 0..case.repetitions {
                restore(&job, &initial(scenario)?)?;
                repetitions.push(capture::capture(
                    &exe,
                    &case
                        .reference_args
                        .iter()
                        .map(Into::into)
                        .collect::<Vec<_>>(),
                    &env,
                    &job,
                    &job,
                    timeout,
                )?);
            }
            captures.insert(scenario.id.clone(), repetitions);
        }
        let stable = captures
            .values()
            .all(|v| v.iter().all(|r| capture::compare(&v[0], r).equal));
        let baseline = Baseline {
            schema_version: 1,
            case: case.id.clone(),
            manifest_sha256,
            profile_sha256,
            job_path_sha256,
            compiler_info: info.stdout,
            reference_binary_sha256: digest(&fs::read(&exe)?),
            captures,
        };
        let target = if stable {
            baseline_path
        } else {
            case_dir.join(format!("unstable-{label}.json"))
        };
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?
            .write_all(&serde_json::to_vec_pretty(&baseline)?)?;
        OpenOptions::new().write(true).create_new(true).open(target.with_extension("md"))?.write_all(
            format!("# Reference baseline: {}\n\nStatus: {}. Profile: `{}`.\n\n{} scenarios, {} runs each. The JSON retains every raw byte and process status. Capture equality covers all regular-file bytes before and after each job.\n",case.id,if stable {"stable"} else {"variable; certification blocked"},profile.id,case.scenarios.len(),case.repetitions).as_bytes())?;
        println!(
            "{}",
            serde_json::json!({"schema_version":1,"case":case.id,"status":if stable {"stable_baseline"} else {"variable_reference"},"repetitions":case.repetitions,"scenarios":case.scenarios.len()})
        );
        return Ok(stable);
    }
    let baseline_bytes = fs::read(&baseline_path)?;
    let baseline: Baseline = serde_json::from_slice(&baseline_bytes)?;
    if baseline.schema_version != 1
        || baseline.case != case.id
        || baseline.manifest_sha256 != manifest_sha256
        || baseline.profile_sha256 != profile_sha256
        || baseline.job_path_sha256 != job_path_sha256
    {
        return Err("baseline contract changed; explicit new baseline required".into());
    }
    if baseline.captures.len() != case.scenarios.len() {
        return Err("baseline scenario set changed".into());
    }
    let mut reports = Vec::new();
    let mut pass = true;
    for scenario in &case.scenarios {
        let expected = baseline
            .captures
            .get(&scenario.id)
            .ok_or("required scenario absent from baseline")?;
        if expected.len() != case.repetitions
            || !expected
                .iter()
                .all(|r| capture::compare(&expected[0], r).equal)
        {
            return Err("baseline is incomplete or unstable".into());
        }
        restore(&job, &initial(scenario)?)?;
        let mut port_args: Vec<OsString> = vec!["port".into(), case.id.clone().into()];
        port_args.extend(scenario.port_args.iter().map(Into::into));
        let actual = capture::capture(
            &std::env::current_exe()?,
            &port_args,
            &env,
            &job,
            &job,
            timeout,
        )?;
        let comparison = capture::compare(&expected[0], &actual);
        pass &= comparison.equal;
        reports.push(serde_json::json!({"scenario":scenario.id,"domain":scenario.domain,"comparison":comparison,"actual":actual}));
    }
    let result = serde_json::json!({"schema_version":1,"case":case.id,"status":if pass {"byte_exact"} else {"mismatch"},"scope":"declared corpus and input domain only","profile":profile.id,"profile_sha256":profile_sha256,"baseline_sha256":digest(&baseline_bytes),"rust_binary_sha256":digest(&fs::read(std::env::current_exe()?)?),"scenarios":reports});
    let report_path = case_dir.join(format!(
        "verification-{label}-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&report_path)?
        .write_all(&serde_json::to_vec_pretty(&result)?)?;
    let mut markdown = format!(
        "# Verification: {}\n\nStatus: {}. Profile: `{}`.\n\nScope: declared corpus and input domain only. Baseline SHA-256: `{}`.\n",
        case.id,
        if pass { "byte-exact" } else { "mismatch" },
        profile.id,
        digest(&baseline_bytes)
    );
    for scenario in result["scenarios"]
        .as_array()
        .ok_or("missing scenario reports")?
    {
        markdown.push_str(&format!(
            "\n## {}\n\n{}\n\nEqual: {}.\n",
            scenario["scenario"].as_str().unwrap_or(""),
            scenario["domain"].as_str().unwrap_or(""),
            scenario["comparison"]["equal"]
        ));
        for difference in scenario["comparison"]["differences"]
            .as_array()
            .ok_or("missing comparisons")?
        {
            markdown.push_str(&format!("\n- `{}`: expected {} bytes; actual {} bytes; first offset {}; expected hex `{}`; actual hex `{}`.\n",difference["artifact"].as_str().unwrap_or(""),difference["expected_len"],difference["actual_len"],difference["first_diff_offset"],difference["expected_context_hex"].as_str().unwrap_or(""),difference["actual_context_hex"].as_str().unwrap_or("")));
        }
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(report_path.with_extension("md"))?
        .write_all(markdown.as_bytes())?;
    println!(
        "{}",
        serde_json::json!({"schema_version":1,"case":case.id,"status":result["status"],"scenario_count":case.scenarios.len(),"profile":profile.id})
    );
    Ok(pass)
}

pub fn port(args: &[String]) -> Result<i32> {
    match args.first().map(String::as_str) {
        Some("formatter") if args.len() == 1 => crate::formatter::run(),
        Some("sales") if args.len() == 1 => crate::sales::run(),
        Some("fxsort") if args.len() == 1 => crate::fxsort::run(),
        _ => Err("unknown or unreviewed port".into()),
    }
}
