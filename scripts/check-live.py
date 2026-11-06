"""Run debug/release live corpus checks; create baselines only with an explicit flag."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, root):
    print(" ".join(str(part) for part in command), flush=True)
    subprocess.run([str(part) for part in command], cwd=root, check=True, timeout=600)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--sources", required=True, type=Path)
    parser.add_argument("--runs", required=True, type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument("--create-baselines", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    runs, sources, profile_path = args.runs.resolve(), args.sources.resolve(), args.profile.resolve()
    required = json.loads((root / "cases/required.json").read_text())
    if not required or len(set(required)) != len(required):
        raise ValueError("required case registry must be nonempty and unique")
    executable = "c2r.exe" if os.name == "nt" else "c2r"
    common = ["--profile", profile_path, "--sources", sources, "--runs", runs, "--label", args.label]
    run(["cargo", "build", "--locked"], root)
    if args.create_baselines:
        for case in required:
            run([root / "target/debug" / executable, "baseline", case, *common], root)
    baselines = {case:runs / case / f"baseline-{args.label}.json" for case in required}
    before = {case:digest(path) for case,path in baselines.items()}
    reports = []
    for mode in ("debug", "release"):
        if mode == "release":
            run(["cargo", "build", "--locked", "--release"], root)
        old = set(runs.glob(f"*/verification-{args.label}-*.json"))
        run([root / f"target/{mode}" / executable, "verify", "--all", *common], root)
        fresh = set(runs.glob(f"*/verification-{args.label}-*.json")) - old
        if len(fresh) != len(required):
            raise ValueError("verification did not produce exactly one report per required case")
        for path in sorted(fresh):
            report = json.loads(path.read_text())
            manifest_path = root / "cases" / f"{report['case']}.json"
            manifest = json.loads(manifest_path.read_text())
            if report["status"] != "byte_exact":
                raise ValueError("non-passing case in live result")
            reports.append({
                "case":report["case"], "mode":mode, "status":report["status"],
                "source_revision":manifest["source_revision"], "sources":manifest["sources"],
                "manifest_sha256":digest(manifest_path),
                "baseline_sha256":report["baseline_sha256"],
                "rust_binary_sha256":report["rust_binary_sha256"],
                "scenarios":[{"id":s["scenario"], "domain":s["domain"],
                              "equal":s["comparison"]["equal"],
                              "stdout_bytes":len(s["actual"]["stdout"]),
                              "stderr_bytes":len(s["actual"]["stderr"]),
                              "stdout_sha256":hashlib.sha256(bytes(s["actual"]["stdout"])).hexdigest(),
                              "stderr_sha256":hashlib.sha256(bytes(s["actual"]["stderr"])).hexdigest(),
                              "status":s["actual"]["status"],
                              "files":{name:{"bytes":len(data), "sha256":hashlib.sha256(bytes(data)).hexdigest()}
                                       for name,data in s["actual"]["after"].items()}}
                             for s in report["scenarios"]],
            })
        if {case:digest(path) for case,path in baselines.items()} != before:
            raise ValueError("verification modified reference captures")
    error_probe_checked = False
    if args.create_baselines:
        # A missing port entry point is a launcher failure, not a byte mismatch.
        probe = runs / f"infrastructure-probe-{args.label}"
        (probe / "cases").mkdir(parents=True)
        case = json.loads((root / "cases/formatter.json").read_text())
        case["id"] = "unimplemented"
        for source in case["sources"]:
            if source["root"] == "repository":
                destination = probe / source["path"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(root / source["path"], destination)
        (probe / "cases/unimplemented.json").write_text(json.dumps(case),encoding="utf-8")
        probe_flags = ["--profile",profile_path,"--sources",sources,"--runs",probe / "runs","--label",args.label]
        run([root / "target/debug" / executable,"baseline","unimplemented",*probe_flags],probe)
        expected = probe / "runs/unimplemented" / f"baseline-{args.label}.json"
        expected_hash = digest(expected)
        failure = subprocess.run([str(v) for v in [root / "target/debug" / executable,"verify","unimplemented",*probe_flags]],cwd=probe,capture_output=True,timeout=120)
        if failure.returncode != 2 or json.loads(failure.stderr)["status"] != "infrastructure_error" or digest(expected) != expected_hash:
            raise ValueError("Rust port infrastructure failure was misclassified or altered the baseline")
        (probe / "result.json").write_text(json.dumps({"exit":failure.returncode,"stdout":failure.stdout.decode(),"stderr":failure.stderr.decode(),"baseline_unchanged":True}),encoding="utf-8")
        error_probe_checked = True
    profile = json.loads(profile_path.read_text())
    summary = {"schema_version":1, "claim":"Byte-exact for each declared corpus and input domain under the recorded execution profile; finite file corpora are not universal equivalence.",
               "profile":profile["id"], "reference_architecture":profile["reference_architecture"],
               "profile_file_manifest_sha256":hashlib.sha256(json.dumps(profile["files"],sort_keys=True).encode()).hexdigest(),
               "profile_sha256":digest(profile_path), "compiler_info_sha256":profile["compiler_info_sha256"],
               "reference_captures_unchanged":True, "rust_port_error_classification_checked":error_probe_checked,"results":reports}
    output = runs / f"summary-{args.label}-{time.time_ns()}.json"
    output.write_text(json.dumps(summary,indent=2) + "\n",encoding="utf-8",newline="\n")
    lines = ["# Live verification", "", summary["claim"], "", f"Profile: `{summary['profile']}`; reference architecture `{summary['reference_architecture']}`.", "", "| Case | Build | Scenarios | Result |", "| --- | --- | ---: | --- |"]
    lines.extend(f"| {r['case']} | {r['mode']} | {len(r['scenarios'])} | {r['status']} |" for r in reports)
    lines += ["", "All reference-capture file hashes were unchanged after both verification builds.", "", "The formatter corpus exhausts all 10,000 valid PIC 99V99 values twice. Other file corpora are finite examples, including declared malformed and missing-file cases. See case manifests for exact domains and any reference adaptation."]
    output.with_suffix(".md").write_text("\n".join(lines)+"\n",encoding="utf-8",newline="\n")
    print(f"Live checks passed; summary retained as {output.name}")


if __name__ == "__main__":
    main()
