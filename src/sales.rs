//! Byte-preserving port of Tyler Bro's `project2.cbl` admitted from commit
//! `ece4ed7efcbcc9a232318956cf9765d036a5b18f` (MIT). The execution profile fixes
//! `COB_CURRENT_DATE`; report bytes follow the retained GnuCOBOL captures.
// SPDX-License-Identifier: MIT
// Copyright 2017 Tyler. Modified port copyright 2026 Saimon Islam.
use crate::Result;
use std::fs;
use std::io::{self, Write};

const RECORD_LEN: usize = 60;
const REPORT_LEN: usize = 71;
const ITEM_CENTS_MODULUS: i128 = 100_000_000;
const TOTAL_CENTS_MODULUS: i128 = 1_000_000_000;
const TOTAL_QUANTITY_MODULUS: i128 = 1_000_000;
const MISSING_INPUT_ERROR: &[u8] = b"libcob: error: READ/START not allowed, file not open for input (status = 47) for file SOURCE-FILE ('PR2FA17_txt' => PR2FA17.txt)\r\nlibcob: warning: implicit CLOSE of REPORT-FILE ('SALES REPORT.txt')\r\n";

#[derive(Default)]
struct Group {
    quantity: Option<i128>,
    cents: Option<i128>,
}

/// Runs from the current directory using `PR2FA17.txt` and writes `SALES REPORT.txt`.
/// `COB_CURRENT_DATE` is required and uses GnuCOBOL's `YYYY/MM/DD ...` override syntax.
/// OUTPUT: the reference return code; missing input writes the reference header and diagnostic.
pub fn run() -> Result<i32> {
    let date = date_from_environment()?;
    let input = match fs::read("PR2FA17.txt") {
        Ok(input) => input,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut output = headings(&date);
            output.push(b'\n');
            fs::write("SALES REPORT.txt", output)?;
            let mut stderr = io::stderr().lock();
            stderr.write_all(MISSING_INPUT_ERROR)?;
            stderr.flush()?;
            return Ok(1);
        }
        Err(error) => return Err(error.into()),
    };
    let output = report(&input, &date)?;
    fs::write("SALES REPORT.txt", output)?;
    Ok(0)
}

/// Produces the report for raw line-sequential file bytes.
/// INPUT: `input` is a sequence of records; each input record is `PIC X(60)` and short
/// records are space padded. `date` is six ASCII digits in `YYMMDD` order.
/// OUTPUT: exact 71-byte report records with observed line-sequential advances and page bytes.
pub fn report(input: &[u8], date: &[u8; 6]) -> Result<Vec<u8>> {
    if !date.iter().all(u8::is_ascii_digit) {
        return Err(invalid_data(
            "date must be six ASCII digits in YYMMDD order",
        ));
    }
    let mut out = headings(date);
    let mut group = Group::default();
    let mut final_quantity = None;
    let mut final_cents = None;
    let mut previous_id = [b' '; 3];
    let mut grouping_flag = true;
    let mut line_num = 10;
    let mut proper_spacing = 1;

    for bytes in records(input) {
        let record = padded(bytes);
        let product_id: [u8; 3] = record[30..33].try_into().expect("fixed product id field");
        let mut product_name: [u8; 14] =
            record[38..52].try_into().expect("fixed product name field");
        if product_id == previous_id {
            product_name = [b' '; 14];
            grouping_flag = true;
        } else if grouping_flag && previous_id != [b' '; 3] {
            grouping_flag = false;
            let (total, quantity, cents) = total_record(group.quantity, group.cents);
            write_record(&mut out, 2, &total);
            proper_spacing = 3;
            final_quantity = add_optional(final_quantity, quantity, TOTAL_QUANTITY_MODULUS)?;
            final_cents = add_optional(final_cents, cents, TOTAL_CENTS_MODULUS)?;
            group.quantity = Some(0);
            group.cents = Some(0);
        } else {
            grouping_flag = false;
        }

        let quantity = numeric(&record[52..55])?;
        let cost_cents = numeric(&record[55..60])?;
        let cents = quantity
            .checked_mul(cost_cents)
            .ok_or_else(|| invalid_data("item sales multiplication overflow"))?
            % ITEM_CENTS_MODULUS;
        let detail = detail_record(&product_name, &product_id, &record[5..30], quantity, cents);
        write_record(&mut out, proper_spacing, &detail);
        proper_spacing = 1;
        group.quantity = Some(add_mod(
            group.quantity.unwrap_or(0),
            quantity,
            TOTAL_QUANTITY_MODULUS,
        )?);
        group.cents = Some(add_mod(
            group.cents.unwrap_or(0),
            cents,
            TOTAL_CENTS_MODULUS,
        )?);
        previous_id = product_id;

        line_num += 1;
        if line_num == 55 {
            out.push(0x0c);
            out.extend([b' '; REPORT_LEN]);
            headings_after_page(&mut out, date);
            proper_spacing = 1;
            line_num = 10;
        }
    }

    let (total, quantity, cents) = total_record(group.quantity, group.cents);
    write_record(&mut out, 2, &total);
    final_quantity = add_optional(final_quantity, quantity, TOTAL_QUANTITY_MODULUS)?;
    final_cents = add_optional(final_cents, cents, TOTAL_CENTS_MODULUS)?;
    write_record(&mut out, 3, &final_quantity_record(final_quantity));
    write_record(&mut out, 2, &final_cents_record(final_cents));
    out.push(b'\n');
    Ok(out)
}

fn headings(date: &[u8; 6]) -> Vec<u8> {
    let mut out = Vec::new();
    headings_after_page(&mut out, date);
    out
}

fn headings_after_page(out: &mut Vec<u8>, date: &[u8; 6]) {
    let mut one = vec![b' '; 41];
    one[34..41].copy_from_slice(b"DR.CHEB");
    write_record(out, 1, &one);

    let mut two = vec![b' '; 71];
    two[9..11].copy_from_slice(&date[2..4]);
    two[11] = b'/';
    two[12..14].copy_from_slice(&date[4..6]);
    two[14..17].copy_from_slice(b"/20");
    two[17..19].copy_from_slice(&date[0..2]);
    two[26..50].copy_from_slice(b"SALES SPECULATION REPORT");
    two[68..71].copy_from_slice(b"TSB");
    write_record(out, 1, &two);

    let mut three = vec![b' '; 71];
    three[17..21].copy_from_slice(b"PROD");
    three[32..40].copy_from_slice(b"CUSTOMER");
    three[53..56].copy_from_slice(b"QTY");
    three[65..70].copy_from_slice(b"SALES");
    write_record(out, 3, &three);

    let mut four = vec![b' '; 71];
    four[2..14].copy_from_slice(b"PRODUCT NAME");
    four[18..20].copy_from_slice(b"ID");
    four[34..38].copy_from_slice(b"NAME");
    four[52..56].copy_from_slice(b"SOLD");
    four[65..70].copy_from_slice(b"VALUE");
    write_record(out, 1, &four);
    write_record(out, 1, &[b' '; REPORT_LEN]);
}

fn write_record(out: &mut Vec<u8>, advance: usize, record: &[u8]) {
    out.extend(std::iter::repeat_n(b'\n', advance));
    out.extend_from_slice(&record[..record.len().min(REPORT_LEN)]);
    out.resize(out.len() + REPORT_LEN.saturating_sub(record.len()), b' ');
}

fn total_record(
    quantity: Option<i128>,
    cents: Option<i128>,
) -> (Vec<u8>, Option<i128>, Option<i128>) {
    let mut line = vec![b' '; REPORT_LEN];
    line[32..38].copy_from_slice(b"TOTAL:");
    let qty = match quantity {
        Some(value) => integer_field(value, 6, 3),
        None => vec![b' '; 6],
    };
    line[50..56].copy_from_slice(&qty);
    line[58..71].copy_from_slice(&currency(cents.unwrap_or(0), 7, cents.is_none()));
    (line, quantity, cents)
}

fn detail_record(
    name: &[u8; 14],
    product_id: &[u8; 3],
    customer: &[u8],
    quantity: i128,
    cents: i128,
) -> Vec<u8> {
    let mut line = Vec::with_capacity(REPORT_LEN);
    line.push(b' ');
    line.extend_from_slice(name);
    line.extend_from_slice(b"  ");
    line.extend_from_slice(product_id);
    line.extend_from_slice(b"   ");
    line.extend_from_slice(customer);
    line.extend_from_slice(b"    ");
    line.extend(integer_field(quantity, 4, 3));
    line.extend_from_slice(b"     ");
    line.extend(amount(cents, 6));
    line
}

fn final_quantity_record(quantity: Option<i128>) -> Vec<u8> {
    let mut line = vec![b' '; REPORT_LEN];
    line[30..48].copy_from_slice(b"TOTAL AMOUNT SOLD:");
    let quantity = match quantity {
        Some(value) => edited_integer(value, 7, 0, true, false),
        None => b"     ,   ".to_vec(),
    };
    line[62..71].copy_from_slice(&quantity);
    line
}

fn final_cents_record(cents: Option<i128>) -> Vec<u8> {
    let mut line = vec![b' '; REPORT_LEN];
    line[27..48].copy_from_slice(b"TOTAL VALUE OF SALES:");
    line[54..71].copy_from_slice(&currency(cents.unwrap_or(0), 10, false));
    line
}

fn amount(cents: i128, integer_width: usize) -> Vec<u8> {
    edited_integer(cents / 100, integer_width, 0, true, false)
        .into_iter()
        .chain([
            b'.',
            b'0' + ((cents / 10) % 10) as u8,
            b'0' + (cents % 10) as u8,
        ])
        .collect()
}

fn currency(cents: i128, integer_width: usize, preserve_zero_commas: bool) -> Vec<u8> {
    std::iter::once(b'$')
        .chain(edited_integer(
            cents / 100,
            integer_width,
            0,
            true,
            preserve_zero_commas,
        ))
        .chain([
            b'.',
            b'0' + ((cents / 10) % 10) as u8,
            b'0' + (cents % 10) as u8,
        ])
        .collect()
}

fn integer_field(value: i128, width: usize, mandatory: usize) -> Vec<u8> {
    let digits = value.rem_euclid(10_i128.pow(width as u32)).to_string();
    let padded = format!("{digits:0>width$}");
    let mut seen = false;
    padded
        .bytes()
        .enumerate()
        .map(|(index, digit)| {
            if index >= width - mandatory || digit != b'0' {
                seen = true;
            }
            if seen || digit != b'0' { digit } else { b' ' }
        })
        .collect()
}

fn edited_integer(
    value: i128,
    width: usize,
    mandatory: usize,
    commas: bool,
    preserve_zero_commas: bool,
) -> Vec<u8> {
    let padded = format!("{:0>width$}", value.rem_euclid(10_i128.pow(width as u32)));
    let mut seen = false;
    let mut out = Vec::with_capacity(width + (width - 1) / 3);
    for (index, digit) in padded.bytes().enumerate() {
        if commas && index > 0 && (width - index).is_multiple_of(3) {
            out.push(if seen || (value == 0 && preserve_zero_commas) {
                b','
            } else {
                b' '
            });
        }
        if index >= width - mandatory || digit != b'0' {
            seen = true;
        }
        out.push(if seen || digit != b'0' { digit } else { b' ' });
    }
    out
}

fn records(input: &[u8]) -> impl Iterator<Item = &[u8]> {
    let end = input.strip_suffix(b"\n").map_or(input.len(), <[u8]>::len);
    input[..end]
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty() || !input.is_empty())
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
}

fn padded(line: &[u8]) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];
    let n = line.len().min(RECORD_LEN);
    record[..n].copy_from_slice(&line[..n]);
    record
}

fn numeric(bytes: &[u8]) -> Result<i128> {
    bytes.iter().try_fold(0_i128, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => i128::from(*byte - b'0'),
            b' ' => 0,
            _ => return Err(invalid_data("input contains a nonnumeric PIC 9 byte")),
        };
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .ok_or_else(|| invalid_data("numeric PIC 9 field overflow"))
    })
}

fn add_optional(total: Option<i128>, value: Option<i128>, modulus: i128) -> Result<Option<i128>> {
    match (total, value) {
        (None, None) => Ok(None),
        (left, right) => Ok(Some(add_mod(
            left.unwrap_or(0),
            right.unwrap_or(0),
            modulus,
        )?)),
    }
}

fn add_mod(left: i128, right: i128, modulus: i128) -> Result<i128> {
    Ok(left
        .checked_add(right)
        .ok_or_else(|| invalid_data("PIC arithmetic overflow"))?
        .rem_euclid(modulus))
}

fn date_from_environment() -> Result<[u8; 6]> {
    let value = std::env::var("COB_CURRENT_DATE")
        .map_err(|_| invalid_data("COB_CURRENT_DATE is required by the admitted profile"))?;
    let bytes = value.as_bytes();
    if bytes.len() < 10
        || bytes[4] != b'/'
        || bytes[7] != b'/'
        || !bytes[..4]
            .iter()
            .chain(&bytes[5..7])
            .chain(&bytes[8..10])
            .all(u8::is_ascii_digit)
    {
        return Err(invalid_data("COB_CURRENT_DATE must start with YYYY/MM/DD"));
    }
    Ok([bytes[2], bytes[3], bytes[5], bytes[6], bytes[8], bytes[9]])
}

fn invalid_data(message: &str) -> Box<dyn std::error::Error + Send + Sync> {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}
