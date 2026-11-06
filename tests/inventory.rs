use cobol_to_rust::{digest, inventory};
use std::{fs, io::Write, path::Path};

fn zip(path: &Path, files: &[(&str, &[u8])]) {
    let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in files {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn retains_duplicate_provenance_and_reconciles_serialized_owner_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-work/inventory")
        .join(std::process::id().to_string());
    fs::create_dir_all(&root).unwrap();
    zip(
        &root.join("source.zip"),
        &[
            ("X/COBOL_Files/A_case/one.cob", b"CALL 'external'\0\xff"),
            ("X/COBOL_Files/A_case/two.cob", b"CALL 'external'\0\xff"),
            ("X/COBOL_Files/A_case/fields.cpy", b"01 FIELD PIC X."),
            ("X/COBOL_Files/A_case/README", b"not a source"),
        ],
    );
    zip(&root.join("catalogue.xlsx"), &[
        ("xl/sharedStrings.xml", br#"<sst><si><t>Username</t></si><si><t>RepoName</t></si><si><t>License Name</t></si></sst>"#),
        ("xl/worksheets/sheet1.xml", br#"<worksheet><sheetData>
        <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c></row>
        <row r="2"><c r="A2" t="inlineStr"><is><t>NamedUser(login=&quot;A&quot;)</t></is></c><c r="B2" t="inlineStr"><is><t>case</t></is></c><c r="C2" t="inlineStr"><is><t>Unverified label</t></is></c></row>
        <row r="3"><c r="A3" t="inlineStr"><is><t>B</t></is></c><c r="B3" t="inlineStr"><is><t>absent</t></is></c></row>
        </sheetData></worksheet>"#),
    ]);
    let result = inventory::scan(&root.join("source.zip"), &root.join("catalogue.xlsx")).unwrap();
    assert_eq!(
        (
            result.archive_files,
            result.program_count,
            result.copybook_count,
            result.distinct_source_contents
        ),
        (4, 2, 1, 2)
    );
    assert!(result.unmatched_archive_folders.is_empty());
    assert_eq!(result.catalogue[0].archive_folders, ["A_case"]);
    assert!(result.catalogue[1].archive_folders.is_empty());
    assert_eq!(
        result.duplicates[&digest(b"CALL 'external'\0\xff")].len(),
        2
    );
    assert_eq!(
        result
            .sources
            .iter()
            .find(|s| s.path.ends_with("one.cob"))
            .unwrap()
            .lexical_dependencies,
        ["external-calls"]
    );
    fs::remove_dir_all(root).unwrap();
}
