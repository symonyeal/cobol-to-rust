//! Archive provenance and catalogue reconciliation. Lexical findings are admission leads, not parsing.
use crate::{Result, digest};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    path::Path,
};
use zip::ZipArchive;

#[derive(Serialize)]
pub struct Source {
    pub path: String,
    pub repo: String,
    pub bytes: usize,
    pub sha256: String,
    pub kind: &'static str,
    pub lexical_dependencies: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct Entry {
    pub row: usize,
    pub owner: String,
    pub repository: String,
    pub license_label: String,
    pub archive_folders: Vec<String>,
}

#[derive(Serialize)]
pub struct Inventory {
    pub schema_version: u32,
    pub archive_sha256: String,
    pub catalogue_sha256: String,
    pub archive_files: usize,
    pub program_count: usize,
    pub copybook_count: usize,
    pub distinct_source_contents: usize,
    pub repositories: BTreeSet<String>,
    pub sources: Vec<Source>,
    pub duplicates: BTreeMap<String, Vec<String>>,
    pub catalogue: Vec<Entry>,
    pub unmatched_archive_folders: Vec<String>,
}

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn login(owner: &str) -> &str {
    owner
        .strip_prefix("NamedUser(login=\"")
        .and_then(|s| s.strip_suffix("\")"))
        .unwrap_or(owner)
}

fn xml(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String> {
    let mut text = String::new();
    zip.by_name(name)?.read_to_string(&mut text)?;
    Ok(text)
}

/// Read the first worksheet (the collection's Sheet1), including shared and inline strings.
fn catalogue(data: &[u8]) -> Result<Vec<Entry>> {
    let mut zip = ZipArchive::new(Cursor::new(data))?;
    let strings = match xml(&mut zip, "xl/sharedStrings.xml") {
        Ok(s) => roxmltree::Document::parse(&s)?
            .descendants()
            .filter(|n| n.has_tag_name("si"))
            .map(|n| {
                n.descendants()
                    .filter(|t| t.has_tag_name("t"))
                    .filter_map(|t| t.text())
                    .collect::<String>()
            })
            .collect::<Vec<_>>(),
        Err(_) => vec![],
    };
    let sheet = xml(&mut zip, "xl/worksheets/sheet1.xml")?;
    let doc = roxmltree::Document::parse(&sheet)?;
    let mut rows = Vec::new();
    for row in doc.descendants().filter(|n| n.has_tag_name("row")) {
        let mut cells = BTreeMap::new();
        for cell in row.children().filter(|n| n.has_tag_name("c")) {
            let col: String = cell
                .attribute("r")
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            let value = cell
                .descendants()
                .find(|n| n.has_tag_name("v"))
                .and_then(|n| n.text())
                .unwrap_or("");
            let value = match cell.attribute("t") {
                Some("s") => strings
                    .get(value.parse::<usize>()?)
                    .ok_or("bad shared string index")?
                    .clone(),
                Some("inlineStr") => cell
                    .descendants()
                    .filter(|n| n.has_tag_name("t"))
                    .filter_map(|n| n.text())
                    .collect(),
                _ => value.to_owned(),
            };
            cells.insert(col, value);
        }
        rows.push((row.attribute("r").unwrap_or("0").parse::<usize>()?, cells));
    }
    let header = rows.first().ok_or("empty catalogue")?.1.clone();
    let col = |label: &str| {
        header
            .iter()
            .find_map(|(c, v)| (v == label).then_some(c.clone()))
            .ok_or_else(|| format!("catalogue lacks {label}"))
    };
    let owner = col("Username")?;
    let repo = col("RepoName")?;
    let license = col("License Name")?;
    Ok(rows
        .into_iter()
        .skip(1)
        .filter(|(_, r)| r.get(&repo).is_some_and(|v| !v.is_empty()))
        .map(|(row, r)| Entry {
            row,
            owner: r.get(&owner).cloned().unwrap_or_default(),
            repository: r.get(&repo).cloned().unwrap_or_default(),
            license_label: r.get(&license).cloned().unwrap_or_default(),
            archive_folders: vec![],
        })
        .collect())
}

pub fn scan(archive: &Path, workbook: &Path) -> Result<Inventory> {
    let data = fs::read(archive)?;
    let workbook = fs::read(workbook)?;
    let mut zip = ZipArchive::new(Cursor::new(&data))?;
    let mut sources = Vec::new();
    let mut repositories = BTreeSet::new();
    let mut archive_files = 0;
    let mut hashes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.is_dir() {
            continue;
        }
        archive_files += 1;
        let name = file.name().to_string();
        let parts: Vec<_> = name.split('/').collect();
        let repo = if parts.len() > 3 && parts[1] == "COBOL_Files" {
            parts[2].to_string()
        } else {
            continue;
        };
        repositories.insert(repo.clone());
        let ext = Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let kind = match ext.as_str() {
            "cpy" => "copybook",
            "cob" | "cbl" | "cobol" | "cobol85" => "program",
            _ => continue,
        };
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let sha256 = digest(&bytes);
        hashes.entry(sha256.clone()).or_default().push(name.clone());
        let text = String::from_utf8_lossy(&bytes).to_uppercase();
        let lexical_dependencies = [
            ("COPY ", "copybooks"),
            ("CALL ", "external-calls"),
            ("EXEC SQL", "embedded-sql"),
            ("EXEC CICS", "cics"),
            ("SCREEN SECTION", "screens"),
            ("INDEXED", "indexed-files"),
            ("SORT ", "sort"),
            ("MERGE ", "merge"),
            ("ACCEPT ", "external-input"),
            ("SELECT ", "file-assignments"),
        ]
        .into_iter()
        .filter_map(|(needle, label)| text.contains(needle).then_some(label))
        .collect();
        sources.push(Source {
            path: name,
            repo,
            bytes: bytes.len(),
            sha256,
            kind,
            lexical_dependencies,
        });
    }
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    let mut catalogue = catalogue(&workbook)?;
    let mut unmatched_archive_folders = Vec::new();
    for folder in &repositories {
        let (owner, repo) = folder
            .split_once('_')
            .ok_or("archive folder lacks owner separator")?;
        let matches: Vec<_> = catalogue
            .iter_mut()
            .filter(|r| norm(login(&r.owner)) == norm(owner) && norm(&r.repository) == norm(repo))
            .collect();
        if matches.len() == 1 {
            matches
                .into_iter()
                .next()
                .unwrap()
                .archive_folders
                .push(folder.clone());
        } else {
            unmatched_archive_folders.push(folder.clone());
        }
    }
    Ok(Inventory {
        schema_version: 1,
        archive_sha256: digest(&data),
        catalogue_sha256: digest(&workbook),
        archive_files,
        program_count: sources.iter().filter(|s| s.kind == "program").count(),
        copybook_count: sources.iter().filter(|s| s.kind == "copybook").count(),
        distinct_source_contents: hashes.len(),
        repositories,
        sources,
        duplicates: hashes.into_iter().filter(|(_, v)| v.len() > 1).collect(),
        catalogue,
        unmatched_archive_folders,
    })
}

impl Inventory {
    pub fn markdown(&self) -> String {
        format!(
            "# Source inventory\n\nArchive SHA-256: `{}`\n\n{} file entries; {} programs; {} copybooks; {} distinct source contents; {} repositories.\n\n{} catalogue entries; {} matched entries; {} absent from archive; {} unmatched/ambiguous archive folders.\n\n{} duplicate-content groups. Original paths and every duplicate occurrence are retained in the JSON register. Lexical dependencies are leads; they do not establish buildability or licence permission.\n",
            self.archive_sha256,
            self.archive_files,
            self.program_count,
            self.copybook_count,
            self.distinct_source_contents,
            self.repositories.len(),
            self.catalogue.len(),
            self.catalogue
                .iter()
                .filter(|r| !r.archive_folders.is_empty())
                .count(),
            self.catalogue
                .iter()
                .filter(|r| r.archive_folders.is_empty())
                .count(),
            self.unmatched_archive_folders.len(),
            self.duplicates.len()
        )
    }
}
