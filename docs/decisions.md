# Decision journal

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
