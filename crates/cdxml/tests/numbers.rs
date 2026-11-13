use bondline_cdxml::{ObjectId, parse_integer};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn object_ids_follow_the_published_u32_and_unassigned_zero_rules()
-> Result<(), Box<dyn std::error::Error>> {
    // CDX-INTRO's bond example uses ID 29 and endpoint references 21 and 22.
    for (spelling, expected) in [
        ("29", 29),
        ("21.00", 21),
        ("00022", 22),
        ("4294967295.0", u32::MAX),
    ] {
        let id: ObjectId = spelling.parse()?;
        assert_eq!(id.get(), expected);
        assert!(id.is_assigned());
    }
    for spelling in ["0", "000.00", "-0", " +0 "] {
        let id: ObjectId = spelling.parse()?;
        assert_eq!(id.get(), 0);
        assert!(!id.is_assigned());
    }
    for spelling in ["-1", "4294967296", "1.00000000000000000001", "21 22", ""] {
        assert!(spelling.parse::<ObjectId>().is_err());
    }
    Ok(())
}

#[test]
fn integral_decimal_values_agree_with_the_independent_python_decimal_oracle()
-> Result<(), Box<dyn std::error::Error>> {
    let vectors = [
        ("255", 255),
        ("255.0", 255),
        ("255.00", 255),
        (" \t+000255.000\r\n", 255),
        ("-128.00", -128),
        ("0.0", 0),
        ("-0.000", 0),
        ("9223372036854775807.00", i64::MAX),
        ("-9223372036854775808.000", i64::MIN),
    ];
    for (spelling, expected) in vectors {
        assert_eq!(parse_integer(spelling)?, expected);
    }
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut oracle = Command::new(python)
        .args([
            "-c",
            r#"
import sys,decimal
stream=sys.stdin.buffer
for _ in range(9):
    spelling=stream.read(int(stream.readline())).decode('ascii')
    expected=int(stream.readline())
    numerator,denominator=decimal.Decimal(spelling).as_integer_ratio()
    assert denominator==1 and numerator==expected
assert not stream.read()
"#,
        ])
        .env("PYTHONIOENCODING", "ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = oracle.stdin.take().ok_or("missing Decimal stdin")?;
    for (spelling, expected) in vectors {
        writeln!(input, "{}", spelling.len())?;
        input.write_all(spelling.as_bytes())?;
        writeln!(input, "{expected}")?;
    }
    drop(input);
    let result = oracle.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn fractional_overflow_and_unsupported_spellings_are_never_rounded_into_integers()
-> Result<(), Box<dyn std::error::Error>> {
    for spelling in [
        "",
        " ",
        "+",
        "-",
        ".0",
        "1.",
        "1.00.0",
        "1.00000000000000000001",
        "-0.00000000000000000001",
        "1e0",
        "0x10",
        "NaN",
        "inf",
        "9223372036854775808",
        "-9223372036854775809",
        "1\u{a0}",
    ] {
        assert!(parse_integer(spelling).is_err(), "accepted {spelling:?}");
    }
    assert_eq!(parse_integer(&format!("{}1.000", "0".repeat(70_000)))?, 1);
    Ok(())
}
