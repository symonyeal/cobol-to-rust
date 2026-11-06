# Phase two: packed decimal command-line arithmetic

Status: local implementation and validation complete; clean Windows CI pending.

## Selection

The selected candidate is `scottieinor/simple-calc-langs`, `COBOL/cobol-calc.cbl`, represented in the existing source register as `scottieinor_simple-calc-langs/cobol-calc.cbl`. Revision `2e7fd52165700a5dad20af91255c49f72b29212f` matches the registered archive source exactly. The source SHA-256 is `b390906e9044d7779b602f5040196d69b7f9e167bf207f69208d3028f743ab4e`; its revision-matched MIT licence SHA-256 is `c4dd62c54d4ab4e8090517d1cb0720638f7b44ee9a641cf1924c7870dd07ee53`. Retrieval metadata is pinned in `sources.json`, with Scott Stevenson's notice retained in `licenses/` and the Rust port.

The inspected source accepts two operands and an operator from command-line arguments. Both operands are unsigned four-digit `PACKED-DECIMAL` fields. The result is an unsigned eight-digit display field. The program prints the accepted operands, performs addition, subtraction, multiplication or division, and prints the result. An unsupported operator prints an apology and then displays the result field without having assigned a calculation to it.

This adds numeric input conversion into packed decimal, unsigned destination behavior and division truncation to the workbench. Those details matter when replacing COBOL calculations: an ordinary integer parser or a mathematically reasonable handling of negative results can change the observable output. Missing and malformed arguments also reach runtime conversion rules before arithmetic starts.

The alternatives reviewed included CICS examples with transaction-system dependencies and z390 demonstrations with additional dialect assumptions. This calculator provides a smaller executable boundary for the new numeric question. Its size limits the integration work while still requiring evidence about behavior that the existing formatter, sort and report cases do not establish.

## Reference contract

Keep the source unchanged. If the pinned compiler cannot admit it without a source change, retain the failed build and prepare an exact proposed change for approval before proceeding. The earlier IBM fxsort adaptation remains a separate approved reference, with its original and adapted hashes and attribution preserved.

Establish at least three identical reference runs for every declared scenario before writing the Rust implementation. Retain new captures under new labels; preserve all earlier baselines. Compare raw stdout, stderr, status and complete regular-file snapshots at job boundaries. The calculator declares no file I/O, so unchanged fixture files and absence of new files are the relevant file effects.

The corpus must distinguish ordinary operations, zero and maximum operands, negative subtraction results, fractional division, zero division, operator truncation, numeric input conversion, unsupported operators and missing arguments. Exact admitted scenarios and their observed outputs belong in the case manifest and retained evidence, rather than being inferred from this selection rationale.

## Observations and implementation

No reference source change was necessary: `-x -fixed -std=default` compiled the unchanged source with status 0 and empty diagnostics. Before implementing Rust, `packed-v1` retained three identical runs for 28 scenarios. Those captures exposed a false coverage assumption: the MinGW executable expands `*` against filenames before `ACCEPT`. A sentinel fixture made the operator `s` from `sentinel.bin`, and multiplication was never reached. Those captures remain retained. The corrected `packed-v2` has 31 scenarios, three identical runs each, including actual multiplication in empty directories and one-file/two-file wildcard cases. The two-file case also shifts the next operand to a filename.

Observed results include `1 - 9999` displaying `00009998`, `9999 * 9999` displaying `99980001`, `10 / 3` displaying `00000003`, and division by zero displaying zero with no stderr and status 0. Four-digit input overflow retains low digits (`12345` becomes `2345`). Negative operands lose their sign, fractional input truncates, and the tested malformed numeric strings become zero. Unsupported operators print the original apology and then eight zeroes. All output uses CRLF and fixed zero padding. These are compatibility observations, including questionable behavior, rather than arithmetic corrections.

The Rust implementation uses bounded unsigned integers for visible values; it does not recreate packed storage internally. The only workbench extension is an optional per-scenario reference argv override. Existing manifests remain byte-identical and use their original case-level defaults. No additional dependencies were needed. The complete corpus is executable directly from `cases/calculator.json`: its strings and hex file bytes are the deterministic input definition, with no random generation or expected-output manufacture.

The implementation effort consisted of one port module, dispatcher registration, argv support, a 31-scenario manifest, a regression test, provenance, and a rerunnable mutation check. The unexpected work was identifying native wildcard expansion and separating real arithmetic coverage from it. Elapsed engineering time and runtime performance were not measured. This supports feasibility for this bounded numeric program; it does not estimate costs for a production estate.

## Validation

The full local gate created separate `packed-final` captures for all four ports, with three identical reference repetitions for each of 56 scenarios. Debug and release comparisons passed raw stdout, stderr, status and before/after regular-file bytes. [Local evidence](evidence/phase-two-local.json) records all eight case/build results and unchanged capture hashes. [The preservation audit](evidence/phase-one-preserved.json) compares original baseline and manifest hashes to the original published ledger; all three remain unchanged.

The deliberate defect changed subtraction from `a.abs_diff(b)` to `a.saturating_sub(b)`. Verification returned 1, rejecting `negative-result` at stdout offset 31; the baseline SHA remained unchanged. The exact source was restored, rebuilt and reverified successfully. [The diagnostic](evidence/calculator-rejected-defect.json) and `scripts/check-calculator-defect.py` retain the experiment. The Windows workflow repeats the mutation, restores the port and then runs the entire debug/release gate again. Clean-run results will be recorded here after artifact inspection.

## Limits

This case does not establish general packed-decimal storage compatibility. Internal memory and packed bytes are outside the observation contract; the visible behavior is command-line conversion, arithmetic and display. Signed fields, implied decimal scales, rounding clauses, size-error handlers, copybooks, indexed files, transaction processing and mainframe runtimes remain outside this case. The tested corpus is finite. Performance is not measured.

General `ACCEPT` grammar is not established: repeated signs, arbitrary comma placement, huge inputs, non-ASCII and embedded NUL are unvalidated. Wildcard support is limited to the tested ASCII literal `*` in empty directories or with the declared ordinary filenames; other masks, quoted patterns, hidden/system files, directories, mixed-case ordering/ties and Unicode filenames are unvalidated. The source opens no files: file evidence establishes sentinel preservation and no added/deleted files for these fixtures, not indexed-file capability. Broader migration cases still need admission, provenance and their own corpora. Existing observation limits (timestamps, permissions, transient files and surviving children) remain.
