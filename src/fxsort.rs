//! Port of IBM fxsort (Apache-2.0); reference metadata adaptation in docs/fxsort-admission.md.
// SPDX-License-Identifier: Apache-2.0
// Copyright 2020 IBM Corp. Modified port copyright 2026 Saimon Islam.
use crate::Result;
use std::{
    fs,
    io::{self, Write},
};

pub const RECORD_BYTES: usize = 80;
pub type Record = [u8; RECORD_BYTES];

/// Fixed-record input: preserve all bytes and pad an incomplete final record with spaces.
/// The pinned runtime's cob_copy_check pads the SORT destination after a short read.
pub fn records(bytes: &[u8]) -> Vec<Record> {
    bytes
        .chunks(RECORD_BYTES)
        .map(|chunk| {
            let mut record = [b' '; RECORD_BYTES];
            record[..chunk.len()].copy_from_slice(chunk);
            record
        })
        .collect()
}

/// GnuCOBOL 3.2 feeds all USING records through its stable sort for SORT and MERGE.
/// Keys are raw character fields, never parsed numeric prices. Equal keys keep submission order.
pub fn outputs(mut input: Vec<Record>) -> (Vec<Record>, Vec<Record>) {
    input.sort_by(|a, b| a[..50].cmp(&b[..50]));
    let merged = input.clone();
    input.sort_by(|a, b| a[50..63].cmp(&b[50..63]));
    (merged, input)
}

pub fn run() -> Result<i32> {
    let mut input = Vec::new();
    for file in ["FXBOSS", "FXMXR", "FXJHS"] {
        match fs::read(file) {
            Ok(bytes) => input.extend(records(&bytes)),
            // The reference SORT/MERGE helper silently skips an input it cannot open.
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    let (merged, sorted) = outputs(input);
    fs::write("FXLSTMRG", merged.as_flattened())?;
    fs::write("FXSORTED", sorted.as_flattened())?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(b"MERGING FILES\r\nSORTING RECORDS\r\nNOW I'M STOPPING\r\n")?;
    stdout.flush()?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_binary_records_and_pads_short_tail() {
        let mut bytes = vec![0; 80];
        bytes.extend([255, 0]);
        let result = records(&bytes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], [0; 80]);
        assert_eq!(&result[1][..3], &[255, 0, b' ']);
        assert_eq!(result[1][79], b' ');
        assert!(records(&[]).is_empty());
    }
    #[test]
    fn sorts_edited_prices_as_characters_and_preserves_tie_order() {
        let record = |name: u8, price: &[u8; 13], tag: u8| {
            let mut record = [b' '; 80];
            record[0] = name;
            record[50..63].copy_from_slice(price);
            record[79] = tag;
            record
        };
        let a = record(b'Z', b"       $12.00", 1);
        let b = record(b'A', b"$000000002.00", 2);
        let c = record(b'A', b"$000000002.00", 3);
        let (merged, sorted) = outputs(vec![a, b, c]);
        assert_eq!(merged, vec![b, c, a]);
        // The raw field beginning with a space sorts before the field beginning with '$'.
        assert_eq!(sorted, vec![a, b, c]);
    }
}
