"""Provision and verify the pinned Windows GnuCOBOL reference profile."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any


SOURCE_URL = "https://downloads.sourceforge.net/project/gnucobol/gnucobol/3.2/gnucobol-3.2_win.zip"
SOURCE_SHA256 = "98e16ec53d1389de9a392f1c565493acd610dc0de15a68af779b0f6d5283b62a"
BINARY_URL = "https://www.arnoldtrembley.com/GC32-VBI-SP1-rename-7z-to-exe.7z"
BINARY_SHA256 = "546523b293cd6a3dabb7ae173af2bee8a11d850db3f91ef3201d8f6b2e4e9d72"
EXTRACTOR_URL = "https://downloads.sourceforge.net/project/sevenzip/7-Zip/26.04/7zr.exe"
EXTRACTOR_SHA256 = "256feca8e274e5da655e2a284fabafd9f554365eb164862089dacd4e8276d282"
PROFILE_ID = "gnucobol-3.2.0-mingw32-vbisam-arnold-trembley"
CURRENT_DATE = "2026/11/06 12:34:56"
SMOKE_SOURCE = """       IDENTIFICATION DIVISION.
       PROGRAM-ID. PROFILE-SMOKE.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 ACCEPTED-DATE PIC 9(6).
       PROCEDURE DIVISION.
           ACCEPT ACCEPTED-DATE FROM DATE.
           DISPLAY \"ACCEPT-DATE=\" ACCEPTED-DATE.
           DISPLAY \"CURRENT-DATE=\" FUNCTION CURRENT-DATE.
           STOP RUN.
"""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def record(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_logged(
    command: list[str], cwd: Path, log: Path, timeout: int, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[bytes]:
    try:
        result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, check=False, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        record(
            log,
            {
                "command": command,
                "cwd": str(cwd),
                "timeout_seconds": timeout,
                "result": "timeout",
                "stdout_hex": (error.stdout or b"").hex(),
                "stderr_hex": (error.stderr or b"").hex(),
            },
        )
        raise RuntimeError(f"command timed out after {timeout}s; see {log}") from error
    record(
        log,
        {
            "command": command,
            "cwd": str(cwd),
            "timeout_seconds": timeout,
            "returncode": result.returncode,
            "stdout_hex": result.stdout.hex(),
            "stderr_hex": result.stderr.hex(),
        },
    )
    return result


def download(curl: str, url: str, target: Path, expected: str, log: Path) -> None:
    if target.exists():
        actual = sha256(target)
        if actual != expected:
            raise RuntimeError(f"pinned download hash mismatch: {target.name}: {actual}")
        record(log, {"url": url, "artifact": target.name, "sha256": actual, "result": "reused verified file"})
        return

    partial = target.with_name(target.name + ".partial")
    command = [
        curl,
        "--ssl-no-revoke",
        "--location",
        "--fail",
        "--silent",
        "--show-error",
        "--connect-timeout",
        "30",
        "--max-time",
        "300",
        "--output",
        str(partial),
        url,
    ]
    try:
        result = subprocess.run(command, capture_output=True, check=False, timeout=330)
    except subprocess.TimeoutExpired as error:
        record(
            log,
            {
                "command": command,
                "url": url,
                "artifact": target.name,
                "timeout_seconds": 330,
                "result": "timeout",
                "stdout_hex": (error.stdout or b"").hex(),
                "stderr_hex": (error.stderr or b"").hex(),
            },
        )
        raise RuntimeError(f"download timed out for {target.name}; see {log}") from error
    actual = sha256(partial) if partial.exists() else None
    record(
        log,
        {
            "command": command,
            "url": url,
            "artifact": target.name,
            "returncode": result.returncode,
            "stdout_hex": result.stdout.hex(),
            "stderr_hex": result.stderr.hex(),
            "partial_sha256": actual,
        },
    )
    if result.returncode:
        raise RuntimeError(f"download failed for {target.name}; see {log}")
    if actual != expected:
        raise RuntimeError(f"pinned download hash mismatch for {target.name}; partial retained at {partial}")
    partial.replace(target)


def extract_source(archive: Path, destination: Path) -> None:
    expected_root = destination / "gnucobol-3.2_win"
    with zipfile.ZipFile(archive) as package:
        expected: dict[str, str] = {}
        for item in package.infolist():
            name = PurePosixPath(item.filename)
            mode = item.external_attr >> 16
            if (
                "\\" in item.filename
                or ":" in item.filename
                or name.is_absolute()
                or not name.parts
                or name.parts[0] != "gnucobol-3.2_win"
                or ".." in name.parts
                or stat.S_ISLNK(mode)
            ):
                raise RuntimeError(f"unsafe source archive entry: {item.filename}")
            if not item.is_dir():
                expected["/".join(name.parts[1:])] = hashlib.sha256(package.read(item)).hexdigest()
        if expected_root.is_dir():
            current = manifest(expected_root)
            if current != expected:
                raise RuntimeError(f"existing source extraction differs from pinned archive: {expected_root}")
            return
        for item in package.infolist():
            target = destination.joinpath(*PurePosixPath(item.filename).parts)
            if item.is_dir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(package.read(item))
    if not (expected_root / "configure.ac").is_file():
        raise RuntimeError("source archive did not contain the expected GnuCOBOL 3.2 tree")


def manifest(root: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise RuntimeError(f"symlink found in compiler tree: {path}")
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    return result


def environment() -> dict[str, str]:
    return {
        "COB_MAIN_DIR": "${ROOT}\\",
        "COB_CONFIG_DIR": "${ROOT}\\config",
        "COB_COPY_DIR": "${ROOT}\\copy",
        "COB_CFLAGS": '-I"${ROOT}\\include"',
        "COB_LDFLAGS": '-L"${ROOT}\\lib"',
        "COB_LIBRARY_PATH": "${ROOT}\\extras",
        "COB_RUNTIME_CONFIG": "${ROOT}\\config\\runtime.cfg",
        "COB_CURRENT_DATE": CURRENT_DATE,
        "LOCALEDIR": "${ROOT}\\locale",
        "TZ": "UTC0",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": "${ROOT}\\bin;C:\\Windows\\System32;C:\\Windows",
        "SystemRoot": "C:\\Windows",
        "TEMP": "${ROOT}\\..\\tmp",
        "TMP": "${ROOT}\\..\\tmp",
    }


def expanded_environment(values: dict[str, str], root: Path) -> dict[str, str]:
    return {key: value.replace("${ROOT}", str(root)) for key, value in values.items()}


def profile_files_match(profile_path: Path, root: Path, files: dict[str, str]) -> bool:
    if not profile_path.is_file():
        return False
    try:
        previous = json.loads(profile_path.read_text(encoding="utf-8"))
        return previous.get("id") == PROFILE_ID and previous.get("files") == files
    except (OSError, json.JSONDecodeError):
        return False


def verify_or_extract(extractor: Path, archive: Path, root: Path, profile_path: Path) -> None:
    if root.exists():
        existing = manifest(root)
        if profile_files_match(profile_path, root, existing):
            return
        raise RuntimeError(f"existing compiler tree is not a verified profile: {root}")

    renamed = archive.with_name("GC32-VBI-SP1.exe")
    if renamed.exists() and sha256(renamed) != BINARY_SHA256:
        raise RuntimeError(f"unexpected existing self-extracting archive: {renamed}")
    if not renamed.exists():
        shutil.copyfile(archive, renamed)

    staging = root.parent / f"GC32-VBI-extract-{os.getpid()}"
    result = run_logged(
        [str(extractor), "x", str(renamed), "-o" + str(staging), "-y"],
        root.parent,
        root.parent / "logs" / "extract-compiler.json",
        timeout=120,
    )
    if result.returncode:
        raise RuntimeError("7-Zip extraction failed; see logs/extract-compiler.json")
    if not (staging / "bin" / "cobc.exe").is_file() or not (staging / "bin" / "gcc.exe").is_file():
        raise RuntimeError("archive did not produce the expected cobc.exe and gcc.exe")
    staging.rename(root)


def build_profile(reference_env: Path, root: Path) -> dict[str, Any]:
    profile: dict[str, Any] = {
        "schema_version": 1,
        "id": PROFILE_ID,
        "platform": "x86_64-windows",
        "reference_architecture": "i686-windows",
        "root": str(root),
        "cobc": "bin/cobc.exe",
        "files": manifest(root),
        "environment": environment(),
        "compiler_info_sha256": "",
    }
    env = expanded_environment(profile["environment"], root)
    logs = reference_env / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    work = logs / "smoke"
    work.mkdir(parents=True, exist_ok=True)
    compiler = root / "bin" / "cobc.exe"

    info = run_logged([str(compiler), "--info"], work, logs / "compiler-info.json", timeout=30, env=env)
    (logs / "compiler-info.stdout.bin").write_bytes(info.stdout)
    (logs / "compiler-info.stderr.bin").write_bytes(info.stderr)
    if info.returncode or info.stderr:
        raise RuntimeError("cobc --info failed or wrote stderr; see logs/compiler-info.json")
    profile["compiler_info_sha256"] = hashlib.sha256(info.stdout).hexdigest()

    source = work / "smoke-date.cob"
    source.write_text(SMOKE_SOURCE, encoding="ascii", newline="\r\n")
    executable = work / "smoke-date.exe"
    build_args = ["-x", "-fixed", "-std=default", "-o", str(executable), str(source)]
    build = run_logged([str(compiler), *build_args], work, logs / "smoke-build.json", timeout=120, env=env)
    (logs / "smoke-build.stdout.bin").write_bytes(build.stdout)
    (logs / "smoke-build.stderr.bin").write_bytes(build.stderr)
    if build.returncode or build.stderr or not executable.is_file():
        raise RuntimeError("COBOL compile/link smoke failed; see logs/smoke-build.json")

    run = run_logged([str(executable)], work, logs / "smoke-run.json", timeout=30, env=env)
    (logs / "smoke-run.stdout.bin").write_bytes(run.stdout)
    (logs / "smoke-run.stderr.bin").write_bytes(run.stderr)
    expected = re.compile(rb"^ACCEPT-DATE=261106\r?\nCURRENT-DATE=20261106123456\d{2}\+0000\r?\n$")
    if run.returncode or run.stderr or not expected.fullmatch(run.stdout):
        raise RuntimeError("COBOL runtime/date smoke failed; see logs/smoke-run.json")
    return profile


def main() -> int:
    if os.name != "nt":
        raise RuntimeError("this reference profile is Windows-only")
    parser = argparse.ArgumentParser(description="Download, pin, and smoke-test the GnuCOBOL 3.2 Windows reference profile.")
    parser.add_argument("--reference-env", type=Path, required=True, help="external reference-env directory for downloaded artifacts and profile.json")
    args = parser.parse_args()
    reference_env = args.reference_env.resolve()
    reference_env.mkdir(parents=True, exist_ok=True)
    (reference_env / "tmp").mkdir(exist_ok=True)
    logs = reference_env / "logs"
    logs.mkdir(exist_ok=True)
    curl = shutil.which("curl.exe")
    if not curl:
        raise RuntimeError("curl.exe is required; Windows 10/11 curl supports the pinned downloads")

    source_zip = reference_env / "gnucobol-3.2_win.zip"
    author_package = reference_env / "GC32-VBI-SP1-rename-7z-to-exe.7z"
    extractor = reference_env / "7zr-26.04.exe"
    download(curl, SOURCE_URL, source_zip, SOURCE_SHA256, logs / "download-source.json")
    download(curl, BINARY_URL, author_package, BINARY_SHA256, logs / "download-compiler.json")
    download(curl, EXTRACTOR_URL, extractor, EXTRACTOR_SHA256, logs / "download-extractor.json")
    extract_source(source_zip, reference_env)

    root = reference_env / "GC32-VBI"
    profile_path = reference_env / "profile.json"
    verify_or_extract(extractor, author_package, root, profile_path)
    profile = build_profile(reference_env, root)
    record(profile_path, profile)
    record(
        logs / "provision-summary.json",
        {
            "status": "verified",
            "profile": PROFILE_ID,
            "platform": profile["platform"],
            "reference_architecture": profile["reference_architecture"],
            "pinned_file_count": len(profile["files"]),
            "compiler_info_sha256": profile["compiler_info_sha256"],
            "smoke_build_args": ["-x", "-fixed", "-std=default"],
            "smoke_output_hex": (logs / "smoke-run.stdout.bin").read_bytes().hex(),
        },
    )
    print(json.dumps({"status": "verified", "profile": PROFILE_ID, "pinned_files": len(profile["files"]), "compiler_info_sha256": profile["compiler_info_sha256"]}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, zipfile.BadZipFile) as error:
        print(f"provision-reference: {error}", file=sys.stderr)
        raise SystemExit(1) from error
