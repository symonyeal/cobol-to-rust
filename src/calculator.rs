//! Port of Scott Stevenson's cobol-calc.cbl (MIT), pinned in docs/sources.json.
// SPDX-License-Identifier: MIT
// Copyright (c) 2014 Scott Stevenson. Rust port copyright 2026 Saimon Islam.
use crate::Result;
use std::{fs, io::Write};

/// Convert the declared ASCII argument subset to unsigned PIC 9(4).
/// Sign is discarded; integer digits overflow modulo 10000; fractional digits truncate.
/// The malformed arguments in this corpus convert to zero; general ACCEPT grammar
/// (including repeated signs and arbitrary separator placement) is unvalidated.
fn operand(text: &str) -> u32 {
    let text = text.trim_matches(' ').trim_start_matches(['+', '-']);
    let mut parts = text.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    if parts.next().is_some()
        || whole.is_empty() && fraction.is_none()
        || !whole.bytes().all(|b| b.is_ascii_digit() || b == b',')
        || fraction.is_some_and(|s| !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return 0;
    }
    whole
        .bytes()
        .filter(u8::is_ascii_digit)
        .fold(0, |n, b| (n * 10 + u32::from(b - b'0')) % 10000)
}

/// ASCII literal `*` expansion reproduces the MinGW startup behavior exercised by
/// this corpus. Other wildcard patterns, hidden files and non-ASCII argv are unvalidated.
fn arguments(args: &[String]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for arg in args {
        if arg == "*" {
            let mut names = fs::read_dir(".")?
                .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
                .collect::<std::io::Result<Vec<_>>>()?;
            names.sort_by_key(|s| s.to_ascii_lowercase());
            if !names.is_empty() {
                result.extend(names);
                continue;
            }
        }
        result.push(arg.clone());
    }
    Ok(result)
}

pub fn run(args: &[String]) -> Result<i32> {
    let args = arguments(args)?;
    let a = operand(args.first().map_or("", String::as_str));
    let op = args.get(1).and_then(|s| s.bytes().next()).unwrap_or(b' ');
    let b = operand(args.get(2).map_or("", String::as_str));
    let total = match op {
        b'+' => a + b,
        b'-' => a.abs_diff(b),
        b'*' => a * b,
        b'/' => a.checked_div(b).unwrap_or(0),
        _ => 0,
    };
    let mut output = format!("{a:04} {} {b:04}\r\n", char::from(op));
    if !b"+-*/".contains(&op) {
        output.push_str("Sorry I do not know that operator\r\n");
    }
    output.push_str(&format!("The total is: {total:08}\r\n"));
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(0)
}
