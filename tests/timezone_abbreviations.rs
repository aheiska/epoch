use std::process::Command;

#[test]
fn eest_cli_uses_fixed_offset_in_winter() {
    let output = Command::new(env!("CARGO_BIN_EXE_epoch"))
        .args(["-z", "EEST", "1767225600"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "2026-01-01T03:00:00+03:00");
}

#[test]
fn eest_cli_reverse_uses_fixed_offset_in_winter() {
    let output = Command::new(env!("CARGO_BIN_EXE_epoch"))
        .args(["--tz", "eest", "-r", "-u", "s", "2026-01-01T03:00:00"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "1767225600");
}

#[test]
fn ambiguous_abbreviation_cli_reports_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_epoch"))
        .args(["-z", "EST", "0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("Ambiguous timezone abbreviation"));
}