# Reproduce the evidence

Use Windows x64, Python 3.14, `curl.exe`, and the Rust toolchain pinned in `rust-toolchain.toml`. External source, compiler and run directories are ignored by Git. No credentials are needed to retrieve the public reference sources.

```text
python scripts/provision-reference.py --reference-env external/reference
python scripts/fetch-sources.py external/sources
python scripts/prepare-fxsort.py external/sources
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python scripts/generate-corpus.py --check
python scripts/check-live.py --create-baselines --profile external/reference/profile.json --sources external/sources --runs runs --label first
```

The final command builds each reference, captures every scenario three times from identical initial files, checks stability, and runs debug and release Rust verification. It hashes the reference captures before and after both verification builds. With explicit baseline creation it also builds an isolated, deliberately unimplemented port case and requires infrastructure exit 2. The summary JSON/Markdown and complete captures remain in `runs`. The [Windows workflow](../.github/workflows/verify.yml) runs the same live commands in a separate job from the fixture tests and retains evidence as an artifact.

For another verification, omit `--create-baselines` and retain the same profile, paths, sources, manifest and label. Each verification and summary gets a new filename. Changing any part of the reference contract requires a new explicit baseline label; the prior captures remain in place. To examine baseline changes, compare the raw JSON captures by artifact and byte, and retain both bundles. A new passing capture does not erase an earlier mismatch.

## Reproduce a mismatch

After establishing a formatter baseline, temporarily replace the formatter's comma byte (`b','`) with a semicolon (`b';'`) in `src/formatter.rs`, rebuild, and run `c2r verify formatter` with the same existing baseline arguments. The gate returns 1 and identifies stdout byte offset 2, comma `2c` versus semicolon `3b`, without modifying the reference capture. Restore the source and verify again. This experiment was run for the initial release; its report is retained in [rejected-defect.json](evidence/rejected-defect.json).

The mismatch JSON includes the full actual capture plus a comparison. Its Markdown companion identifies the artifact, lengths, first differing offset and bounded hexadecimal context. Do not trim or normalize output to resolve a mismatch. Trace the source field or runtime operation responsible, correct the port, and compare against the existing capture.

An infrastructure error returns 2. Examples include a changed compiler file, a missing source, a changed manifest, a failed build, a timeout, or an absent required case. A blocked case returns 1 with its admission blockers. Neither is a passing parity result. `c2r assess CASE` reports source hashes, build requirements and semantic risks before a run.

## Admit another source

The fourth case is the unchanged Scott Stevenson calculator. `cases/calculator.json` is its complete deterministic input corpus: argv bytes are ASCII strings and initial file bytes are hexadecimal. Its 31 scenarios include actual multiplication in empty directories, plus explicit MinGW literal-star expansion with one and two files. Scenario-level `reference_args` overrides the case-level default; old manifests omit it and retain their fingerprints.

After a full baseline/verification run, reproduce its negative control and restored checks with:

```text
python scripts/check-calculator-defect.py --profile external/reference/profile.json --sources external/sources --runs runs --label first
python scripts/check-live.py --profile external/reference/profile.json --sources external/sources --runs runs --label first
```

The defect script requires a new result filename for each label, rebuilds with saturating subtraction, requires exit 1 and the `negative-result` stdout mismatch, checks the baseline hash, restores the exact source in a `finally` block, rebuilds and verifies again. It retains the mismatch capture and `runs/calculator-defect-first.json`. The workflow runs this experiment and then rechecks all ports in debug and release. See [phase two](phase-two.md) for the finite claim boundaries.

Keep its original bytes outside the repository. Verify its licence and exact revision, pin retrieval hashes, and document any source adaptation before compiling it. Enumerate external calls, copybooks, file organization and every external input. Trace paragraph fall-through and scope, field widths and scales, assignment and editing rules, and EOF/error behavior.

Declare initial fixture bytes and domain descriptions in a case manifest. Inputs, intermediates and final files must remain observable at capture boundaries; programs requiring unobserved transient artifacts or surviving child processes are blocked. Establish repeated stable reference observations before implementing the port, preserve any original defects, then review the port and add the case to `cases/required.json`. The registry cannot be empty or contain duplicate case IDs.

No automated gate establishes equivalence outside the declared domain and corpus. Exhaustive formatter coverage applies to its 10,000 valid numeric values; the file cases are finite, deterministic corpora.
