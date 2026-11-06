use cobol_to_rust::{Result, inventory, workbench};
use std::{env, fs, path::Path};

fn option<'a>(args: &'a [String], key: &str) -> Result<&'a str> {
    let i = args
        .iter()
        .position(|a| a == key)
        .ok_or_else(|| format!("missing {key}"))?;
    args.get(i + 1)
        .map(String::as_str)
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| format!("missing value for {key}").into())
}

fn run(args: &[String]) -> Result<bool> {
    match args.first().map(String::as_str) {
        Some("inventory") => {
            let result = inventory::scan(
                Path::new(option(args, "--archive")?),
                Path::new(option(args, "--catalogue")?),
            )?;
            let out = Path::new(option(args, "--out")?);
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(out, serde_json::to_vec_pretty(&result)?)?;
            let report = result.markdown();
            fs::write(out.with_extension("md"), &report)?;
            print!("{report}");
            Ok(result.unmatched_archive_folders.is_empty())
        }
        Some("assess") => workbench::assess(args.get(1).ok_or("missing case")?),
        Some("baseline") | Some("verify") => workbench::execute(args),
        Some("port") => std::process::exit(workbench::port(&args[1..])?),
        _ => {
            eprintln!(
                "c2r inventory --archive ZIP --catalogue XLSX --out JSON\nc2r assess CASE\nc2r baseline CASE --profile JSON --sources DIR --runs DIR --label LABEL\nc2r verify CASE|--all --profile JSON --sources DIR --runs DIR --label LABEL"
            );
            Ok(false)
        }
    }
}

fn main() {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!(
                "{}",
                serde_json::json!({"schema_version":1,"status":"infrastructure_error","error":e.to_string()})
            );
            std::process::exit(2);
        }
    }
}
