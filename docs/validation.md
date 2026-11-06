# Validation ledger

## Additional packed-decimal phase (2026-11-06)

The unchanged Scott Stevenson calculator adds 31 finite scenarios to the original 25. The full local gate passed all 56 in debug and release, with three identical reference repetitions per scenario and unchanged captures. [Phase report](phase-two.md), [local evidence](evidence/phase-two-local.json), [negative control](evidence/calculator-rejected-defect.json), and [preserved phase-one hashes](evidence/phase-one-preserved.json) are separate from the original evidence below. Clean Windows confirmation is pending artifact inspection. This demonstrates only the manifest's argv conversion, unsigned arithmetic, display and file-tree behavior under the pinned profile; it does not certify packed storage interoperability or general COBOL numerics.

Results below were obtained on 2026-11-06 using the native Windows profile `gnucobol-3.2.0-mingw32-vbisam-arnold-trembley`. The reference is GnuCOBOL 3.2.0, MinGW GCC 9.2.0, i686; Rust 1.99.1 targets x86_64 Windows. Both architectures are explicit.

## Source register

`c2r inventory` accounted for 8,731 archive file entries, 851 programs, 244 copybooks, 1,079 distinct source contents and 84 repositories. The register reconciled all 84 folders to 84 of 182 catalogue entries, leaving 98 catalogue entries absent from the archive and no unmatched archive folders. Fifteen duplicate-content groups retain every original path.

An independent Python ZIP audit checked integrity and recomputed every one of the 1,095 path/hash pairs against the Rust register. All pairs matched. The selected source and revision-matched licence hashes also matched. The original archive SHA-256 is `4682be35b248ebdb249d94e12845f6ec2a12d4151e3835d4d1f44ea4f2473fa7`; the public [register](source-register.json) retains paths and hashes without local machine paths.

## Local live comparisons

| Program/reference | Scenarios | Reference repetitions | Debug | Release |
| --- | ---: | ---: | --- | --- |
| DGFIPTools formatter, unchanged source | 1 exhaustive domain traversal | 3 | Byte-exact | Byte-exact |
| IBM fxsort, approved metadata adaptation | 10 | 3 each | Byte-exact | Byte-exact |
| Tyler sales report, unchanged source | 14 | 3 each | Byte-exact | Byte-exact |

The full gate ran `scripts/check-live.py` with explicit baseline creation, then reverified against the retained captures after the injected-defect test. It ran `verify --all` in debug and release. All 25 scenarios passed in each build; the hash of every reference-capture file was unchanged by verification. The result projection is [live-windows.json](evidence/live-windows.json), including domains, source and output hashes, compiler identity and profile fingerprints. Complete raw bundles remain in the external run directory.

The formatter traversal covers all 10,000 valid unsigned `PIC 99V99` values, then repeats them in reverse order. Each run compares 100,000 stdout bytes, empty stderr, status 0 and the job file tree. This is exhaustive over the declared valid numeric values under this profile, not over invalid encodings or other runtimes.

The sort claim applies to the [adapted reference](fxsort-admission.md). Its untouched upstream source did not compile under this profile. The owner approved adding two identification-metadata terminators and a modification notice; source logic and data declarations remain unchanged. No result is claimed for an executable built from the untouched original.

The [sales corpus](../cases/sales.json) includes 44/45/46-record page boundaries, repeated pages, 9-digit cents and 6-digit quantity truncation, internal-zero editing, initialized zero, sub-dollar amounts, repeated and singleton groups, partial records and missing input. Existing group-state and uninitialized-field behavior is preserved.

## Instrument checks

- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` passed locally. The suite has 21 passing tests, including the child-process fixture.
- Comparator tests individually reject byte changes, trailing spaces, newline changes, reordered/duplicated records, missing/additional/empty files, changed status/stderr and modified/deleted inputs. Binary NUL, non-UTF-8 bytes, paths containing spaces, concurrent 1 MiB streams and timeout behavior are exercised.
- A full CLI experiment temporarily changed the formatter comma byte to a semicolon. Verification returned 1, reported stdout offset 2 and left the reference hash unchanged. [The retained diagnostic](evidence/rejected-defect.json) records this result. The production source was restored and both build modes passed the full corpus afterward.
- Missing required cases and malformed manifests return infrastructure failure. Blocked cases cannot pass the all-case gate. Original fxsort build failures were retained before its admission adaptation.
- A deliberately absent Rust port entry point returns infrastructure exit 2, with its launcher diagnostic kept out of the parity comparison and the failed capture retained separately. The native probe confirmed the original capture was unchanged; fresh live CI runs exercise this check after the admitted corpus.

## Reproduction and boundaries

The [clean Windows run for commit 1423eda](https://github.com/symonyeal/cobol-to-rust/actions/runs/42294427851) passed both jobs, including fresh compiler/source provisioning and live debug/release parity. Its downloaded raw captures were compared directly with the laptop's captures: all fifty scenario observation pairs have equal stdout, stderr, status and before/after file bytes. Compiler-file and case-manifest fingerprints match; raw compiler-info hashes differ because they include installation paths. [The comparison record](evidence/clean-windows-ci.json) retains both physical profile fingerprints. A local manifest line-ending alignment was recorded through a new explicit baseline label, with earlier captures preserved.

The [Windows workflow](../.github/workflows/verify.yml) separates fixture tests from live reference comparisons. The live job downloads pinned artifacts, verifies hashes, compiles date smoke probes, fetches exact sources, prepares the documented sort adaptation, and runs debug/release comparisons. It uploads raw baseline/verification captures and summaries. Its status for a given commit is visible in [Actions](https://github.com/symonyeal/cobol-to-rust/actions/workflows/verify.yml).

No speed claim is made. File-case coverage is finite. Timestamp, permission and internal-memory equivalence is not measured. The recorder observes regular files at job boundaries; it does not trace transient files removed before a boundary or contain surviving descendant processes. Cases requiring those capabilities remain outside admission. The pinned date override controls the sales source's `ACCEPT ... FROM DATE`; it does not freeze the fractional digits of `FUNCTION CURRENT-DATE`.
