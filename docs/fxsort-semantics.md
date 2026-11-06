# Sort port semantics

The acceptance source is the [explicitly adapted IBM revision](fxsort-admission.md). Its data and procedure declarations are unchanged. It uses six file assignments: inputs `FXBOSS`, `FXMXR`, `FXJHS`; work file `WRK`; intermediate `FXLSTMRG`; final `FXSORTED`. The test corpus is small enough for the pinned runtime's in-memory sort, so it does not exercise disk spills.

Each fixed record is 80 bytes: name 50, edited character price 13, filler 17. The port stores `[u8; 80]`; it does not decode identifiers or convert the edited price into a number. A final incomplete record is padded with spaces in the sort destination. A missing input is silently skipped, matching the observed runtime helper. The input files are retained unchanged.

For this GnuCOBOL 3.2 profile, `cob_file_sort_using_extfh` submits all input records to the same sort machinery for both SORT and MERGE. The intermediate file therefore contains a stable ascending name ordering, even for the intentionally unsorted input in `currency-boundaries`. The final sort compares the 13 price bytes, preserving the intermediate order for tied keys. This is a statement about the pinned runtime and tested domain, not an IBM MERGE equivalence claim.

`character-key-order` contains byte keys whose ordering differs from decimal interpretation. `generated-three-way-merge` supplies 150 seeded, distinguishable records, with equal names and prices across files. Other scenarios cover empty input, one record, leading zeroes, boundary strings, duplicates, a missing file and short records of 79 and one byte.

Both output files, all inputs, stdout, stderr and process status are compared. The three display messages use the captured CRLF bytes; successful runs return 0 with empty stderr, including the missing-input scenario. File metadata and transient writes removed between capture boundaries are outside the observation contract.
