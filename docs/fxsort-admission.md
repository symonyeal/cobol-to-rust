# fxsort compiler admission

The pinned reference compiler accepts the source after two metadata-only sentence terminators are restored. The original upstream source remains unchanged and is retained outside this repository.

## Provenance

- Original: [`IBM/cobol-is-fun` at `12626aa45dd244218c18d67c35ed159a958fadeb`](https://github.com/IBM/cobol-is-fun/commit/12626aa45dd244218c18d67c35ed159a958fadeb), file `my-first-program/fxsort.cbl`.
- Original source SHA-256: `971d185e34000f20e1200e505ec20a6fd1462884318a93f14c92fa02c75572a2`.
- Original license: Apache-2.0, pinned [`LICENSE`](https://github.com/IBM/cobol-is-fun/blob/12626aa45dd244218c18d67c35ed159a958fadeb/LICENSE); local public copy is [`licenses/ibm-apache-2.0.txt`](../licenses/ibm-apache-2.0.txt).
- Adapted external path: `candidates/adapted/fxsort.cbl` beneath the source root supplied to `scripts/prepare-fxsort.py`.
- Adapted SHA-256: `5662b48475500d0404ceb7fc0065fdab36cf9abadf7c810a7d8e49523909d965`.

## Adaptation

The preparation script verifies the original SHA-256 before writing. It adds a terminating period after `PROGRAM-ID. FXSORT` and after `AUTHOR. Jeff Bisti`. It inserts one fixed-format comment stating that metadata periods were added. The notice records the modification for the Apache-2.0 source. Program logic and data declarations are otherwise unchanged. The original archive copy is never overwritten.

Run `python scripts/prepare-fxsort.py SOURCE_ROOT` after fetching sources. `SOURCE_ROOT` must contain `IBM_cobol-is-fun/fxsort.cbl`; the script writes only `candidates/adapted/fxsort.cbl` under that root.

## Compiler evidence

Compiler profile: `gnucobol-3.2.0-mingw32-vbisam-arnold-trembley`. The compiler information bytes matched the pinned profile (`9073ad0b47693fc618680c3d39274e3e9e488b59bb6167be0ccbdee2fb3859f1`). The adapted source compiled with `-x -fixed -std=default` and exit code 0. GnuCOBOL emitted only its warning that the original file has no final newline. The compiled executable is a build observation only; byte-exact baseline capture and live parity remain separate checks.

A clone with only the AUTHOR period failed, as did alternative documented source formats. Adding the PROGRAM-ID terminator as well resolved compiler admission. Probe outputs and hashes are retained with the external source-register work artifacts.
