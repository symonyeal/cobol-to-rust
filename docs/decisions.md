# Decision journal

## 2026-11-06 — Discover an input outside the argument list

**Problem:** The initial calculator corpus labelled three cases as multiplication, but the captured operator was `s`, not `*`.

**Options:** Assume the shell had passed the intended arguments, change the reference build to disable expansion, or treat native startup expansion as observable behavior.

**Choice:** Preserve the unchanged source and compiler flags. Retain `packed-v1` captures, use `packed-v2` for empty-directory arithmetic plus explicit one-file/two-file wildcard cases, and reproduce the tested literal-star expansion in Rust. Add optional per-scenario reference arguments without changing existing case manifests.

**Evidence:** The 28-scenario v1 baseline had three identical runs each. Its sentinel file caused `*` to expand to `sentinel.bin`. The 31-scenario v2 baseline confirms actual multiplication in empty directories and argument shifting with two filenames. The first Rust debug verification matches all 31 scenarios. General Windows wildcard and non-ASCII argument compatibility remain unvalidated.

**Lesson:** A process's input contract can include directory contents even when its source opens no files. Capture actual behavior before naming coverage.

## 2026-11-06 — Reject a mathematically plausible numeric correction

**Problem:** The calculator's unsigned result displays the magnitude of subtraction below zero. Saturating subtraction would silently change that behavior.

**Options:** Correct negative arithmetic, reject such inputs, or preserve the reference's visible result.

**Choice:** Preserve `1 - 9999` as `00009998`. Exercise the alternative as a deliberate defect, then restore the port.

**Evidence:** `scripts/check-calculator-defect.py` changes `a.abs_diff(b)` to `a.saturating_sub(b)`. Local verification returns 1, rejects `negative-result` at stdout offset 31, and confirms the baseline SHA is unchanged. Restored debug verification passes 31 scenarios. The script repeats this experiment in clean Windows CI.

**Lesson:** A familiar arithmetic operation is still a compatibility decision. A passing comparison earns trust by rejecting a concrete, plausible deviation.

These entries preserve consequential choices for engineering review and later LinkedIn posts. Suggested wording is limited to evidence in the repository. Posts are not published by this workbench.

## 2026-11-06 — Define what the migration must preserve

**Decision:** Accept a program revision, execution profile, input domain and corpus together. Compare raw output streams, process status and all regular file bytes at job boundaries.

**Why it matters:** A migration can preserve the calculation and still break the next system through spacing, record order or a changed filename. Compatibility includes those interfaces and existing defects.

**Evidence:** `src/capture.rs` and injected-defect tests in `tests/capture.rs`. The implementation observes boundary snapshots; transient files removed between snapshots are not recorded. Cases requiring such files must remain blocked until step-level observation or tracing is implemented.

**Public wording:** “I started by defining the evidence a migration would need to produce. That includes the bytes surrounding the calculation.” Do not describe finite tests as universal equivalence.

## 2026-11-06 — Treat collection metadata as a lead

**Decision:** Retain every original path and content hash, reconcile the catalogue separately, and verify licence terms at the matching source revision.

**Why it matters:** A spreadsheet describes a collection; it does not establish what was actually collected or which terms cover a file. The sales-report row said “No License,” but its matching upstream revision contains an MIT licence.

**Evidence:** `docs/source-register.json` records 8,731 archive files, 851 programs, 244 copybooks, 1,079 distinct source contents, 84 archived repositories, 182 catalogue entries and 98 absent entries. `docs/sources.json` pins three byte-matched source revisions and their licences.

**Public wording:** “Before translating code, I reconciled the source collection and its provenance. One candidate's catalogue label disagreed with its actual licence.” The archive is not 182 runnable projects.

## 2026-11-06 — Pin the executable environment

**Decision:** Use a documented native Windows GnuCOBOL 3.2/GCC bundle and explicitly distinguish its 32-bit reference architecture from the Rust host architecture. Hash the compiler, runtime, configuration and toolchain files.

**Why it matters:** The compiler version alone does not define file and numeric behavior. The official Windows ZIP turned out to contain source, so a ready-to-run reference required a separate, pinned distribution.

**Evidence:** `docs/reference-profile.md` and the retained profile from provisioning. Build flags and per-case source hashes live in the case manifests. The date override must be tested through the COBOL operation used by the sales report.

**Public wording:** “The execution environment is part of the program's observable behavior. I recorded the actual compiler build and architecture instead of relying on a version label.” No claim of Enterprise COBOL equivalence.

## 2026-11-06 — Establish the measuring instrument first

**Decision:** Keep immutable repeated reference captures separate from verification. Start with the formatter's finite domain before implementing file/report ports.

**Why it matters:** A passing comparison is useful only if it can reject a real defect. The formatter has 10,000 valid input values and a five-byte return field, so it makes a tractable first acceptance target.

**Evidence:** The driver visits every valid value in ascending and descending order, preserving repeated calls to the same subprogram. Baseline creation requires at least three identical runs. Verification cannot invoke baseline creation.

**Public wording:** “For this formatter, I checked every declared valid input and all five returned bytes under one pinned profile.” The live formatter gate passed; this does not extend to invalid input encodings or other compiler profiles.

## 2026-11-06 — Record an admission repair before porting

**Decision:** Keep IBM's original source unchanged. With the owner's approval, create a separately hashed reference that adds periods to the PROGRAM-ID and AUTHOR entries, plus a modification notice. Document both hashes and the failed original build.

**Why it matters:** The unmodified candidate did not compile under the pinned compiler. Changing a reference without recording it would make a parity claim ambiguous. The adapted reference is a separate acceptance unit, even though its procedure and data declarations are unchanged.

**Evidence:** `scripts/prepare-fxsort.py` checks the original hash and produces the exact adapted hash listed in `docs/fxsort-admission.md`. A single AUTHOR terminator was insufficient; both identification entries needed terminators. The adapted source produced three identical runs in each declared scenario before the port was accepted.

**Public wording:** “One candidate needed a metadata-only compiler admission change. I preserved the original, recorded the exact diff and hashes, and named the adapted reference in the result.” Do not call this parity against the untouched original.

## 2026-11-06 — Preserve the observed file algorithm

**Decision:** Match this GnuCOBOL build's behavior for MERGE and edited character keys. Keep the intermediate merge file in the acceptance observations.

**Why it matters:** A source keyword or a business interpretation of a field can suggest an algorithm that differs from the runtime's actual behavior. This runtime submits all USING records to its sort machinery. The price key is compared as raw characters.

**Evidence:** `libcob/fileio.c` in the hash-pinned compiler source archive, functions `cob_file_sort_using_extfh` and `cob_copy_check`; ten live sort scenarios. `character-key-order` uses raw price fields whose byte ordering differs from decimal interpretation. Distinguishable payloads test stable ties across files. Partial records test space padding, and a missing input tests the runtime's silent skip behavior.

**Public wording:** “The sort port follows the pinned runtime's output, including intermediate files and character-key ordering. I tested examples that would distinguish a plausible replacement from the observed behavior.” The result is specific to this runtime and corpus.

## 2026-11-06 — Keep defects and representation states visible

**Decision:** Preserve the sales report's group flag and its uninitialized numeric state. Fix defects in the Rust port when the reference comparison exposes them; leave the reference's behavior intact.

**Why it matters:** A run of singleton product groups does not produce the totals a reader might expect. An uninitialized total also edits differently from a numeric zero. Replacing those states with a convenient default would change the report.

**Evidence:** `sales.rs`, `tests/sales.rs`, the fourteen sales scenarios, and the byte-offset reports retained from mismatched attempts. The expanded corpus exercises destination truncation, quantities with internal zeroes, initialized zero and sub-dollar amounts, and page boundaries. Live comparisons exposed heading-column and comma-suppression mistakes that matching file lengths did not detect.

**Public wording:** “The report migration preserved surprising group behavior and distinguished an empty total from a numeric zero. Exact byte comparisons found formatting defects that a totals-only check would miss.” Corrections to the original report need separate requirements and evidence.

## 2026-11-06 — Test the specific clock operation

**Decision:** Demonstrate the fixed date through `ACCEPT ... FROM DATE`, the operation used by the sales source. Do not claim that the same override freezes every clock function.

**Why it matters:** The runtime override fixed the date and seconds while `FUNCTION CURRENT-DATE` still returned changing fractional digits. A configuration setting alone did not establish determinism.

**Evidence:** The provisioning smoke records both operations. `ACCEPT ... FROM DATE` returned `261106`; repeated sales captures were identical. The variable fractional digits remain documented in `reference-profile.md`.

**Public wording:** “I tested the clock operation the program actually used. The same fixed-time setting did not make a different clock function fully deterministic.” Do not describe the profile as a universal fixed clock.

## 2026-11-06 — Reproduce on a second machine

**Decision:** Run the whole workflow on a clean Windows runner and compare its output hashes with the laptop's results. Keep raw profile fingerprints for each installation and pin repository line endings.

**Why it matters:** `cobc --info` includes installation paths, so its raw hash legitimately differs between machines even when every compiler file is identical. The audit also found a local CRLF manifest that Git checked out as LF. Aligning that file required a new explicit baseline label because manifest fingerprints preserve exact bytes.

**Evidence:** `.gitattributes`, the [clean Windows CI run](https://github.com/symonyeal/cobol-to-rust/actions/runs/42294427851), and `docs/evidence/clean-windows-ci.json`. All 962 compiler-file hashes match. Direct comparison found equal raw streams, status and file bytes across all fifty scenario observation pairs in debug and release. The earlier captures remain retained; no acceptance output was normalized.

**Public wording:** “A clean Windows run reproduced the same program observations using the pinned tool files. The cross-machine audit also exposed a manifest line-ending difference, which I recorded through the baseline process.” Each physical installation still has its own profile fingerprint.

## 2026-11-06 — Add a bounded numeric migration case

**Problem:** The first phase's formatter, sort and report results leave packed-decimal argument conversion and arithmetic behavior untested. A larger source alone would not identify which new capability the workbench can preserve.

**Options:** CICS samples introduce transaction-system dependencies; z390 demonstrations introduce additional dialect assumptions. The registered `scottieinor/simple-calc-langs` calculator has a smaller input/output boundary and declares packed-decimal operands directly.

**Choice:** Select `COBOL/cobol-calc.cbl` for the next admission investigation. Inspect its actual source and revision-matched licence, preserve the source unchanged, and establish repeated reference captures before implementing Rust. This is a selection decision; successful admission and parity remain pending.

**Evidence:** The inspected source declares two unsigned `PIC 9(4) PACKED-DECIMAL` operands, accepts arguments with `ACCEPT ... FROM ARGUMENT-VALUE`, and computes into unsigned `PIC 9(8)` display storage. It includes division and an unsupported-operator path that displays the result without assigning a calculation. `docs/phase-two.md` records the planned scope; the case manifest and retained captures will establish observed behavior.

**Broader lesson:** Choose the next case by the behavior it can test. A bounded program can expose important numeric compatibility questions while keeping the reference executable and its observations reviewable. This case cannot establish copybook, indexed-file or mainframe compatibility, or general packed-decimal storage equivalence.
