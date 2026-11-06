use cobol_to_rust::sales::report;

const DATE: &[u8; 6] = b"261106";

fn record(product_id: &[u8; 3], name: &[u8], quantity: u16, cost_cents: u32) -> Vec<u8> {
    let mut row = vec![b' '; 60];
    row[..5].copy_from_slice(b"00001");
    row[5..30].copy_from_slice(b"CLIENT 00001             ");
    row[30..33].copy_from_slice(product_id);
    let n = name.len().min(14);
    row[38..38 + n].copy_from_slice(&name[..n]);
    row[52..55].copy_from_slice(format!("{quantity:03}").as_bytes());
    row[55..60].copy_from_slice(format!("{cost_cents:05}").as_bytes());
    row.push(b'\n');
    row
}

fn lines(report: &[u8]) -> Vec<&[u8]> {
    report.split(|byte| *byte == b'\n').collect()
}

#[test]
fn preserves_fixed_report_bytes_and_empty_totals() {
    let output = report(b"", DATE).unwrap();
    assert_eq!(output.len(), 583);
    assert!(output.starts_with(
        b"\n                                  DR.CHEB                              \n"
    ));
    assert!(output.windows(10).any(|part| part == b"11/06/2026"));
    assert!(output.ends_with(b"TOTAL VALUE OF SALES:      $             .00\n"));
    assert!(
        lines(&output)
            .iter()
            .filter(|line| !line.is_empty())
            .all(|line| line.len() == 71)
    );
}

#[test]
fn renders_uninitialized_zero_and_35_cent_group_totals_as_captured() {
    let empty = report(b"", DATE).unwrap();
    let empty_total = lines(&empty)
        .into_iter()
        .find(|line| line.starts_with(b"                                TOTAL:"))
        .unwrap();
    assert_eq!(&empty_total[58..71], b"$ ,   ,   .00");

    let initialized_zero = report(&record(b"001", b"ITEM", 1, 0), DATE).unwrap();
    let zero_total = lines(&initialized_zero)
        .into_iter()
        .find(|line| line.starts_with(b"                                TOTAL:"))
        .unwrap();
    assert_eq!(&zero_total[58..71], b"$         .00");

    let thirty_five_cents = report(&record(b"001", b"ITEM", 7, 5), DATE).unwrap();
    let cents_total = lines(&thirty_five_cents)
        .into_iter()
        .find(|line| line.starts_with(b"                                TOTAL:"))
        .unwrap();
    assert_eq!(&cents_total[58..71], b"$         .35");
}

#[test]
fn maps_input_fields_pads_partial_record_and_keeps_decimal_format() {
    let full = record(b"001", b"ITEM 0001", 1, 100);
    let normal = report(&full, DATE).unwrap();
    let mut partial = full.clone();
    partial.remove(59); // remove the last cost digit; the line reader supplies a space
    let padded = report(&partial, DATE).unwrap();
    assert_eq!(normal, padded);
    assert!(
        normal.windows(71).any(|line| line
            == b" ITEM 0001       001   CLIENT 00001                  001           1.00")
    );
    assert!(
        normal.windows(71).any(|line| line
            == b"                                TOTAL:               001  $        1.00")
    );
}

#[test]
fn preserves_singleton_group_state_bug_and_repeated_name_suppression() {
    let mut input = Vec::new();
    input.extend(record(b"001", b"FIRST", 1, 100));
    input.extend(record(b"002", b"SECOND", 1, 200));
    let singleton = report(&input, DATE).unwrap();
    let totals: Vec<_> = lines(&singleton)
        .into_iter()
        .filter(|line| line.starts_with(b"                                TOTAL:"))
        .collect();
    assert_eq!(totals.len(), 1);
    assert!(totals[0].ends_with(b"$        3.00"));

    input.extend(record(b"002", b"SECOND", 1, 200));
    let repeated = report(&input, DATE).unwrap();
    assert!(
        repeated
            .windows(71)
            .any(|line| line.starts_with(b"                 002"))
    );
}

#[test]
fn truncates_pic_destinations_and_inserts_form_feed_after_each_45th_record() {
    let mut input = Vec::new();
    for _ in 0..1_003 {
        input.extend(record(b"001", b"ITEM", 999, 10_000));
    }
    let output = report(&input, DATE).unwrap();
    assert_eq!(output.iter().filter(|byte| **byte == 0x0c).count(), 22);
    assert!(lines(&output).iter().any(|line| line.contains(&0x0c)));

    let overflow = (0..11)
        .flat_map(|_| record(b"001", b"ITEM", 999, 99_999))
        .collect::<Vec<_>>();
    let truncated = report(&overflow, DATE).unwrap();
    let total = lines(&truncated)
        .into_iter()
        .find(|line| line.starts_with(b"                                TOTAL:"))
        .unwrap();
    assert_eq!(&total[50..56], b" 10989");
    assert_eq!(&total[58..71], b"$  988,890.11");
    assert!(
        truncated
            .windows(71)
            .any(|line| line.ends_with(b"   10,989"))
    );
}

#[test]
fn retains_embedded_zeroes_after_first_significant_total_digit() {
    for (rows, last_quantity, expected) in [
        (10, 10, b" 10000".as_slice()),
        (100, 101, b"100001".as_slice()),
    ] {
        let mut input = Vec::new();
        for _ in 0..rows {
            input.extend(record(b"001", b"ITEM", 999, 0));
        }
        input.extend(record(b"001", b"ITEM", last_quantity, 0));
        let output = report(&input, DATE).unwrap();
        let total = lines(&output)
            .into_iter()
            .find(|line| line.starts_with(b"                                TOTAL:"))
            .unwrap();
        assert_eq!(&total[50..56], expected);
    }
}
