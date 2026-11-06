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

**Public wording after live verification:** “For this formatter, I checked every declared valid input and all five returned bytes under one pinned profile.” Until that run passes, say the exhaustive driver is implemented, not that the port is verified.
