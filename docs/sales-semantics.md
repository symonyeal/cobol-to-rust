# Sales report port

`src/sales.rs` is a byte-preserving port of `project2.cbl` from
`tylerbro93/COBOL-SALES-INVENTORY-REPORT-DEMONSTRATION`, source commit
`ece4ed7efcbcc9a232318956cf9765d036a5b18f`. The admitted upstream license is MIT;
the original copyright and license text are retained in `licenses/tylerbro93-mit.txt`.

## Observed behavior represented

- The program reads line-sequential records from `PR2FA17.txt`. Each record maps to 60
  bytes; a short line is space padded, including the observed 59-byte data line. A terminal
  line-feed ends a record and is not part of its fields.
- The report contains 71-byte records. Output uses LF bytes. `WRITE AFTER ADVANCING n`
  contributes `n` line-feed bytes before its record; page advancement contributes `0x0c`
  before a blank 71-byte record. After every 45th detail record, the report starts a new
  heading page. The final file ends with LF.
- Item sales are quantity multiplied by cost in integer cents. The source destinations keep
  their declared digit widths: item cents wrap modulo `10^8`, group/final cents modulo
  `10^9`, and group/final quantity modulo `10^6`. Host arithmetic is checked `i128` before
  applying those destination widths.
- The source's grouping flag behavior is preserved. An ID transition writes a prior group
  total only when a previous repeated ID set the flag back to `YES`; a run of singleton
  product IDs therefore accumulates together until a later eligible transition or EOF.
  Equal consecutive IDs suppress the product name on subsequent detail records.
- `COB_CURRENT_DATE` uses the GnuCOBOL runtime override beginning `YYYY/MM/DD`. The
  admitted Windows profile supplies `2026/11/06 12:34:56`, which produces `11/06/2026`.
  The port requires this admitted-profile variable and does not claim behavior for an
  unconfigured runtime date.
- With no input file, the program still writes the captured header-only report, writes the
  captured GnuCOBOL diagnostic bytes to stderr, and returns status 1. Successful captures
  write no stdout/stderr and return status 0.

## Claim boundary

Parity claims apply to the declared sales manifest scenarios under the pinned Windows
GnuCOBOL profile and date. The Rust parser accepts decimal digits and space padding in the
numeric fields; invalid numeric bytes return an infrastructure error. Behavior outside the
admitted records, profile, and tested scenario set is not established as equivalent.
