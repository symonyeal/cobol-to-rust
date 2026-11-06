# COBOL-to-Rust migration workbench

A Rust CLI for inventorying COBOL sources and checking reviewed batch ports against raw reference observations.

The workbench records stdout, stderr, process status and the complete file tree before and after each job. Comparison preserves spaces, line endings, binary bytes, record order, duplicate records and missing versus empty files. Hashes identify inputs; byte comparison decides equality.

Four reviewed Rust ports pass raw byte comparisons under the pinned Windows profile in debug and release builds:

| Port | Declared coverage |
| --- | --- |
| Formatter | All 10,000 valid `PIC 99V99` values, ascending then descending; 20,000 calls and all 100,000 returned bytes per run |
| IBM sort | 10 file scenarios, using the explicitly documented metadata-only reference adaptation |
| Sales report | 14 file scenarios, including truncation, group transitions and page boundaries |
| Packed-decimal calculator | 31 argv/file-tree scenarios: arithmetic, malformed inputs and literal-star expansion |

The [second phase](docs/phase-two.md) expands the finite corpus from 25 to 56 scenarios. Its local evidence is [retained separately](docs/evidence/phase-two-local.json); the original evidence remains unchanged.

Every reference scenario is repeated three times before acceptance. Verification checks raw streams, status and all regular file bytes, including inputs and the sort's intermediate file. [Validation](docs/validation.md) and [retained result hashes](docs/evidence/live-windows.json) identify the scope. File corpora are finite; inventory and fixture tests alone do not establish parity.

The source register accounts for 851 programs and 244 copybooks across 84 archived repositories. These match 84 of the catalogue's 182 entries; 98 entries are absent from the archive. The four selected sources have revision-matched licence evidence.

```text
cargo build --locked
cargo test --locked
cargo run --locked -- inventory --archive collection.zip --catalogue catalogue.xlsx --out inventory.json
cargo run --locked -- assess formatter
```

For live Windows runs, provision the pinned compiler and sources externally, then use the explicit profile and retained run directory:

```text
python scripts/provision-reference.py --reference-env external/reference
python scripts/fetch-sources.py external/sources
python scripts/prepare-fxsort.py external/sources
cargo run --locked -- baseline formatter --profile external/reference/profile.json --sources external/sources --runs runs --label first
cargo run --locked -- verify formatter --profile external/reference/profile.json --sources external/sources --runs runs --label first
cargo run --locked -- verify --all --profile external/reference/profile.json --sources external/sources --runs runs --label first
```

`python scripts/check-live.py --create-baselines --profile external/reference/profile.json --sources external/sources --runs runs --label acceptance` creates new immutable reference captures, then runs debug and release verification and checks that both leave those captures unchanged. Omit `--create-baselines` when reusing captures. Use a new label when changing the source, corpus, manifest or execution profile.

Verification never regenerates expected output. A changed manifest or execution profile requires an explicit new baseline label. Exit 0 means the requested gate passed; exit 1 means mismatch or blocked admission; exit 2 means infrastructure/configuration failure. The all-case gate fails for missing, blocked or skipped required cases.

Read [the acceptance contract](docs/acceptance.md), [reproduction guide](docs/reproducing.md), [source provenance](THIRD_PARTY.md), [engineering decisions](docs/decisions.md), [pinned course references](docs/engineering-references.md) and [reference provisioning](docs/reference-profile.md). Major decisions retain their evidence and claim boundaries for later process writing.

The workbench covers four reviewed candidates. It does not translate code automatically or establish IBM/mainframe equivalence. Finite corpora support claims only for their declared inputs and execution profile. Timestamps, permissions, internal memory, directory entries without files and transient files removed between observation boundaries are outside the recorded file contract. Programs must terminate their children and write only inside the declared job directory; descendant containment is not implemented.

Original workbench code is MIT licensed; the IBM sort port is Apache-2.0. Ports retain their upstream notices; third-party licences and pinned retrieval information are listed in [THIRD_PARTY.md](THIRD_PARTY.md). Original COBOL sources and compiler distributions stay external.
