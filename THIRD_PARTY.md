# Third-party sources

The COBOL programs remain external to this repository. `scripts/fetch-sources.py DESTINATION` downloads each file from its pinned upstream revision, verifies the recorded SHA-256 hashes for source and license, then writes `<folder>/<filename>` and `<folder>/LICENSE` beneath the destination.

| Program | Upstream revision | License | Upstream license |
| --- | --- | --- | --- |
| `format-number.cob` | [DGFIPTools/Calculer-Moyenne-Cobol `360be06`](https://github.com/DGFIPTools/Calculer-Moyenne-Cobol/commit/360be06d084d6fca3903c5b522537f5413c44977) | MIT | [pinned LICENSE](https://github.com/DGFIPTools/Calculer-Moyenne-Cobol/blob/360be06d084d6fca3903c5b522537f5413c44977/LICENSE) |
| `fxsort.cbl` | [IBM/cobol-is-fun `12626aa`](https://github.com/IBM/cobol-is-fun/commit/12626aa45dd244218c18d67c35ed159a958fadeb) | Apache-2.0 | [pinned LICENSE](https://github.com/IBM/cobol-is-fun/blob/12626aa45dd244218c18d67c35ed159a958fadeb/LICENSE) |
| `project2.cbl` | [tylerbro93/COBOL-SALES-INVENTORY-REPORT-DEMONSTRATION `ece4ed7`](https://github.com/tylerbro93/COBOL-SALES-INVENTORY-REPORT-DEMONSTRATION/commit/ece4ed7efcbcc9a232318956cf9765d036a5b18f) | MIT | [pinned LICENSE](https://github.com/tylerbro93/COBOL-SALES-INVENTORY-REPORT-DEMONSTRATION/blob/ece4ed7efcbcc9a232318956cf9765d036a5b18f/LICENSE) |

The complete license texts for these exact revisions are copied to `licenses/`. License identifiers and hashes are in `docs/sources.json`. The source bytes were admitted by exact SHA-256 match to the listed revision; this attribution does not make any claim about other revisions.

The Rust formatter and sales ports retain the upstream MIT notices. `src/fxsort.rs` is an Apache-2.0 port with a modification notice. Original workbench code uses the root MIT licence; the package combines MIT and Apache-2.0 material. The IBM revision's tree has no NOTICE file.

The executable `fxsort` reference uses the explicitly documented metadata-only [admission adaptation](docs/fxsort-admission.md). Run `python scripts/prepare-fxsort.py DESTINATION` after fetching the original sources. It preserves the original and creates a separately hashed source with two identification terminators and a modification notice; it does not change the procedure or data declarations.
