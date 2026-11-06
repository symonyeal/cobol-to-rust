# COBOL-to-Rust migration workbench

A Rust CLI for inventorying COBOL sources and checking reviewed batch ports against raw reference observations.

The workbench records stdout, stderr, process status and the complete file tree before and after each job. Comparison preserves spaces, line endings, binary bytes, record order, duplicate records and missing versus empty files. Hashes identify inputs; byte comparison decides equality.

Implementation is in progress. The source register accounts for 851 programs and 244 copybooks across 84 archived repositories. These match 84 of the catalogue's 182 entries; 98 entries are absent from the archive. The three selected sources have revision-matched licence evidence. Live migration results are recorded in [validation](docs/validation.md); inventory and fixture tests alone do not establish parity.

```text
cargo build --locked
cargo test --locked
cargo run --locked -- inventory --archive collection.zip --catalogue catalogue.xlsx --out inventory.json
cargo run --locked -- assess formatter
```

For live Windows runs, provision the pinned compiler and sources externally, then use the explicit profile and retained run directory:

```text
python scripts/provision-reference.py external/reference
python scripts/fetch-sources.py external/sources
cargo run --locked -- baseline formatter --profile external/reference/profile.json --sources external/sources --runs runs --label first
cargo run --locked -- verify formatter --profile external/reference/profile.json --sources external/sources --runs runs --label first
cargo run --locked -- verify --all --profile external/reference/profile.json --sources external/sources --runs runs --label first
```

Verification never regenerates expected output. A changed manifest or execution profile requires an explicit new baseline label. Exit 0 means the requested gate passed; exit 1 means mismatch or blocked admission; exit 2 means infrastructure/configuration failure. The all-case gate fails for missing, blocked or skipped required cases.

Read [the acceptance contract](docs/acceptance.md), [source provenance](THIRD_PARTY.md), [engineering decisions](docs/decisions.md) and [reference provisioning](docs/reference-profile.md). Major decisions retain their evidence and claim boundaries for later process writing.

V1 covers three batch/file candidates. It does not translate code automatically or establish IBM/mainframe equivalence. Finite corpora support claims only for their declared inputs and execution profile. Timestamps, permissions, internal memory, directory entries without files and transient files removed between observation boundaries are outside the recorded file contract. Programs must terminate their children and write only inside the declared job directory; descendant containment is not implemented.

Workbench code is MIT licensed. Ports retain their upstream notices; third-party licences and pinned retrieval information are listed in [THIRD_PARTY.md](THIRD_PARTY.md). Original COBOL sources and compiler distributions stay external.
