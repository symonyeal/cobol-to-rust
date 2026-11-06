use std::process::Command;

#[test]
fn preserves_reference_arithmetic_and_argument_conversion() {
    // Raw observations from baseline-packed-v2, unchanged Scott Stevenson source.
    for (args, stdout) in [
        (
            ["1", "-", "9999"],
            "0001 - 9999\r\nThe total is: 00009998\r\n",
        ),
        (
            ["10", "/", "3"],
            "0010 / 0003\r\nThe total is: 00000003\r\n",
        ),
        (
            ["42", "/", "0"],
            "0042 / 0000\r\nThe total is: 00000000\r\n",
        ),
        (
            ["12x", "+", "2"],
            "0000 + 0002\r\nThe total is: 00000002\r\n",
        ),
        (
            ["12345", "+", "1"],
            "2345 + 0001\r\nThe total is: 00002346\r\n",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_c2r"))
            .args(["port", "calculator"])
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, stdout.as_bytes());
        assert!(output.stderr.is_empty());
    }
}
