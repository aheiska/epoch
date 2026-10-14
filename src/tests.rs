use super::*;

#[test]
fn test_timezone_abbreviations() {
    let abbreviations = [
        ("GMT", 0), ("WET", 0), ("WEST", 3600),
        ("CET", 3600), ("CEST", 7200), ("EET", 7200), ("EEST", 10800),
        ("EDT", -14400), ("MDT", -21600), ("PDT", -25200),
        ("AKST", -32400), ("AKDT", -28800), ("HST", -36000),
        ("HKT", 28800), ("JST", 32400), ("KST", 32400), ("NPT", 20700),
        ("AWST", 28800), ("AEST", 36000), ("AEDT", 39600), ("ACDT", 37800),
        ("NZST", 43200), ("NZDT", 46800), ("CHAST", 45900), ("CHADT", 49500),
    ];
    for (name, seconds) in abbreviations {
        let expected = TzConfig::Fixed(FixedOffset::east_opt(seconds).unwrap());
        assert_eq!(name.parse::<TzConfig>().unwrap(), expected, "{name}");
        assert_eq!(format!(" {} ", name.to_lowercase()).parse::<TzConfig>().unwrap(), expected, "{name}");
    }
    for name in ["UTC", "utc", "Z", "z"] {
        assert_eq!(name.parse::<TzConfig>().unwrap(), TzConfig::Utc);
    }
}

#[test]
fn test_ambiguous_timezone_abbreviations() {
    for name in ["CST", "CDT", "IST", "BST", "EST", "MST", "PST", "AST", "ADT", "GST", "SST", "NST", "AMT", "WST", "ACST"] {
        for input in [name.to_string(), format!(" {} ", name.to_lowercase())] {
            let error = input.parse::<TzConfig>().unwrap_err();
            assert!(error.contains("Ambiguous timezone abbreviation"), "{input}: {error}");
            assert!(error.contains("IANA"), "{error}");
        }
    }
    assert!("NOTAZONE".parse::<TzConfig>().is_err());
    assert!(matches!("Europe/Helsinki".parse::<TzConfig>().unwrap(), TzConfig::Named(_)));
    assert!(matches!("US/Eastern".parse::<TzConfig>().unwrap(), TzConfig::Named(_)));
    assert_eq!("+03:00".parse::<TzConfig>().unwrap(), TzConfig::Fixed(FixedOffset::east_opt(10800).unwrap()));
}

#[test]
fn test_abbreviations_never_switch_for_dst() {
    for (name, offset, hour) in [
        ("EEST", "+03:00", "03"),
        ("EET", "+02:00", "02"),
        ("CEST", "+02:00", "02"),
        ("CET", "+01:00", "01"),
    ] {
        let tz = name.parse::<TzConfig>().unwrap();
        for date in ["2026-01-01", "2026-07-01"] {
            let utc = parse_date(&format!("{date}T00:00:00Z"), &TzConfig::Utc, None).unwrap();
            assert_eq!(format_datetime(utc, &tz, None), format!("{date}T{hour}:00:00{offset}"));
            assert_eq!(format_datetime(utc, &tz, Some("%Z")), offset);
            let local = format!("{date}T00:00:00");
            let explicit = format!("{local}{offset}");
            assert_eq!(parse_date(&local, &tz, None).unwrap(), parse_date(&explicit, &TzConfig::Utc, None).unwrap());
        }
    }
    let eest = "EEST".parse::<TzConfig>().unwrap();
    assert!(parse_date("2026-03-29T03:30:00", &eest, None).is_ok());
    assert_eq!(parse_date("2026-01-01T00:00:00Z", &eest, None).unwrap(), parse_date("2026-01-01T00:00:00Z", &TzConfig::Utc, None).unwrap());
}

#[test]
fn test_unit_parsing() {
    assert_eq!("auto".parse::<TimestampUnit>().unwrap(), TimestampUnit::Auto);
    assert_eq!("s".parse::<TimestampUnit>().unwrap(), TimestampUnit::Seconds);
    assert_eq!("ms".parse::<TimestampUnit>().unwrap(), TimestampUnit::Milliseconds);
    assert_eq!("us".parse::<TimestampUnit>().unwrap(), TimestampUnit::Microseconds);
    assert_eq!("ns".parse::<TimestampUnit>().unwrap(), TimestampUnit::Nanoseconds);
}

#[test]
fn test_unit_detection() {
    assert_eq!(detect_unit(1_718_000_000), TimestampUnit::Seconds);
    assert_eq!(detect_unit(1_718_000_000_000), TimestampUnit::Milliseconds);
    assert_eq!(detect_unit(1_718_000_000_000_000), TimestampUnit::Microseconds);
    assert_eq!(detect_unit(1_718_000_000_000_000_000), TimestampUnit::Nanoseconds);
}

#[test]
fn test_parse_timestamp_ms() {
    let dt = parse_timestamp("1718000000000", TimestampUnit::Auto).unwrap();
    assert_eq!(dt.to_rfc3339_opts(SecondsFormat::AutoSi, true), "2024-06-10T06:13:20Z");
}

#[test]
fn test_parse_timestamp_s() {
    let dt = parse_timestamp("1718000000", TimestampUnit::Auto).unwrap();
    assert_eq!(dt.to_rfc3339_opts(SecondsFormat::AutoSi, true), "2024-06-10T06:13:20Z");
}

#[test]
fn test_parse_timestamp_us() {
    let dt = parse_timestamp("1718000000123456", TimestampUnit::Auto).unwrap();
    assert_eq!(dt.to_rfc3339_opts(SecondsFormat::AutoSi, true), "2024-06-10T06:13:20.123456Z");
}

#[test]
fn test_parse_timestamp_subseconds() {
    for (unit, units_per_second, nanos_per_unit, name) in [
        (TimestampUnit::Milliseconds, 1_000_i128, 1_000_000, "milliseconds"),
        (TimestampUnit::Microseconds, 1_000_000, 1_000, "microseconds"),
        (TimestampUnit::Nanoseconds, 1_000_000_000, 1, "nanoseconds"),
    ] {
        for (value, secs, nanos) in [
            (0, 0, 0),
            (1, 0, nanos_per_unit),
            (-1, -1, 1_000_000_000 - nanos_per_unit),
            (units_per_second, 1, 0),
            (-units_per_second, -1, 0),
        ] {
            let dt = parse_timestamp(&value.to_string(), unit).unwrap();
            assert_eq!(dt.timestamp(), secs, "{name}: {value}");
            assert_eq!(dt.timestamp_subsec_nanos(), nanos, "{name}: {value}");
        }
        for value in [i128::MIN, i128::MAX] {
            assert_eq!(parse_timestamp(&value.to_string(), unit).unwrap_err(), format!("Timestamp out of range: {value}"));
        }
        let value = i64::MAX as i128 * units_per_second;
        assert_eq!(parse_timestamp(&value.to_string(), unit).unwrap_err(), format!("Invalid timestamp {name}: {value}"));
    }
}

#[test]
fn test_parse_timestamp_float() {
    let dt = parse_timestamp("1718000000.5", TimestampUnit::Auto).unwrap();
    assert_eq!(dt.to_rfc3339_opts(SecondsFormat::AutoSi, true), "2024-06-10T06:13:20.500Z");
}

#[test]
fn test_timezone_conversion() {
    let dt = parse_timestamp("1718000000000", TimestampUnit::Auto).unwrap();
    let tz_helsinki: TzConfig = "Europe/Helsinki".parse().unwrap();
    let formatted = format_datetime(dt, &tz_helsinki, None);
    assert_eq!(formatted, "2024-06-10T09:13:20+03:00");

    let tz_fixed: TzConfig = "+05:30".parse().unwrap();
    let formatted_fixed = format_datetime(dt, &tz_fixed, None);
    assert_eq!(formatted_fixed, "2024-06-10T11:43:20+05:30");

    let tz_local = TzConfig::Local;
    let formatted_local = format_datetime(dt, &tz_local, None);
    assert!(!formatted_local.is_empty());

    let formatted_rfc2822 = format_datetime(dt, &TzConfig::Utc, Some("rfc2822"));
    assert_eq!(formatted_rfc2822, "Mon, 10 Jun 2024 06:13:20 +0000");
}

#[test]
fn test_custom_format() {
    let dt = parse_timestamp("1718000000000", TimestampUnit::Auto).unwrap();
    let formatted = format_datetime(dt, &TzConfig::Utc, Some("%Y-%m-%d %H:%M:%S"));
    assert_eq!(formatted, "2024-06-10 06:13:20");
}

#[test]
fn test_date_to_timestamp() {
    let tz = TzConfig::Utc;
    let ts = process_item("2024-06-10T06:13:20Z", false, TimestampUnit::Milliseconds, &tz, None).unwrap();
    assert_eq!(ts, "1718000000000");

    let ts_sec = process_item("2024-06-10T06:13:20Z", false, TimestampUnit::Seconds, &tz, None).unwrap();
    assert_eq!(ts_sec, "1718000000");
}

#[test]
fn test_process_item_auto() {
    let tz = TzConfig::Utc;
    // Number to RFC3339
    let out1 = process_item("1718000000000", false, TimestampUnit::Auto, &tz, None).unwrap();
    assert_eq!(out1, "2024-06-10T06:13:20Z");

    // RFC3339 to Timestamp (auto reverse)
    let out2 = process_item("2024-06-10T06:13:20Z", false, TimestampUnit::Milliseconds, &tz, None).unwrap();
    assert_eq!(out2, "1718000000000");
}