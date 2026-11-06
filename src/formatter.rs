//! Port of DGFIPTools format-number.cob (MIT); revision and notice in THIRD_PARTY.md.
// SPDX-License-Identifier: MIT
// Copyright 2016 DGFIPTools. Modified port copyright 2026 Saimon Islam.
use crate::Result;
use std::io::Write;

/// A valid PIC 99V99 input is an unsigned integer number of hundredths, 0..=9999.
/// Return the entire PIC X(5) field, including the leading space for integers below ten.
pub fn format_number(cents: u16) -> Result<[u8; 5]> {
    if cents > 9999 {
        return Err("PIC 99V99 input outside declared domain".into());
    }
    let whole = cents / 100;
    Ok([
        if whole < 10 {
            b' '
        } else {
            b'0' + (whole / 10) as u8
        },
        b'0' + (whole % 10) as u8,
        b',',
        b'0' + ((cents % 100) / 10) as u8,
        b'0' + (cents % 10) as u8,
    ])
}

pub fn run() -> Result<i32> {
    let mut output = Vec::with_capacity(100000);
    for cents in (0..10000).chain((0..10000).rev()) {
        output.extend_from_slice(&format_number(cents)?);
    }
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&output)?;
    stdout.flush()?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_five_bytes_at_field_boundaries() {
        for (cents, bytes) in [
            (0, *b" 0,00"),
            (1, *b" 0,01"),
            (999, *b" 9,99"),
            (1000, *b"10,00"),
            (9999, *b"99,99"),
        ] {
            assert_eq!(format_number(cents).unwrap(), bytes);
        }
        assert!(format_number(10000).is_err());
    }
}
