use chrono::{
    DateTime, FixedOffset, Local, LocalResult, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone,
    Utc,
};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampUnit {
    Auto,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl FromStr for TimestampUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Ok(TimestampUnit::Auto),
            "s" | "sec" | "secs" | "second" | "seconds" => Ok(TimestampUnit::Seconds),
            "ms" | "milli" | "millis" | "millisecond" | "milliseconds" => {
                Ok(TimestampUnit::Milliseconds)
            }
            "us" | "µs" | "micro" | "micros" | "microsecond" | "microseconds" => {
                Ok(TimestampUnit::Microseconds)
            }
            "ns" | "nano" | "nanos" | "nanosecond" | "nanoseconds" => {
                Ok(TimestampUnit::Nanoseconds)
            }
            other => Err(format!(
                "Invalid unit '{}'. Supported units: auto, s, ms, us, ns",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TzConfig {
    Utc,
    Local,
    Named(chrono_tz::Tz),
    Fixed(FixedOffset),
}

impl FromStr for TzConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let lower = trimmed.to_lowercase();

        if lower == "utc" || lower == "z" || lower == "gmt" {
            return Ok(TzConfig::Utc);
        }

        if lower == "local" {
            return Ok(TzConfig::Local);
        }

        // Try fixed offset format: +HH:MM, -HH:MM, +HHMM, -HHMM, +HH, -HH
        if let Some(offset) = parse_fixed_offset(trimmed) {
            return Ok(TzConfig::Fixed(offset));
        }

        // Try exact IANA timezone lookup
        if let Ok(tz) = chrono_tz::Tz::from_str(trimmed) {
            return Ok(TzConfig::Named(tz));
        }

        // Try case-insensitive lookup in chrono_tz variants
        let normalized = trimmed.replace(' ', "_").to_lowercase();
        for variant in chrono_tz::TZ_VARIANTS {
            if variant.name().to_lowercase() == normalized {
                return Ok(TzConfig::Named(variant));
            }
        }

        Err(format!(
            "Unknown timezone '{}'. Examples: UTC, local, Europe/Helsinki, America/New_York, +02:00, -05:00",
            trimmed
        ))
    }
}

fn parse_fixed_offset(s: &str) -> Option<FixedOffset> {
    if !s.starts_with('+') && !s.starts_with('-') {
        return None;
    }

    let sign = if s.starts_with('-') { -1 } else { 1 };
    let rest = &s[1..];

    let (hours, mins) = if rest.contains(':') {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let h: i32 = parts[0].parse().ok()?;
        let m: i32 = parts[1].parse().ok()?;
        (h, m)
    } else if rest.len() == 4 {
        let h: i32 = rest[..2].parse().ok()?;
        let m: i32 = rest[2..].parse().ok()?;
        (h, m)
    } else if rest.len() <= 2 {
        let h: i32 = rest.parse().ok()?;
        (h, 0)
    } else {
        return None;
    };

    if hours < 0 || hours > 23 || mins < 0 || mins > 59 {
        return None;
    }

    let total_secs = sign * (hours * 3600 + mins * 60);
    FixedOffset::east_opt(total_secs)
}

/// Auto-detect the timestamp unit based on integer magnitude.
/// < 10^11 (~year 5140 in seconds, ~1973 in ms) => Seconds
/// < 10^14 (~year 5140 in ms) => Milliseconds
/// < 10^17 (~year 5140 in us) => Microseconds
/// >= 10^17 => Nanoseconds
pub fn detect_unit(val: i128) -> TimestampUnit {
    let abs_val = val.abs();
    if abs_val < 100_000_000_000 {
        TimestampUnit::Seconds
    } else if abs_val < 100_000_000_000_000 {
        TimestampUnit::Milliseconds
    } else if abs_val < 100_000_000_000_000_000 {
        TimestampUnit::Microseconds
    } else {
        TimestampUnit::Nanoseconds
    }
}

/// Parse a numeric string (integer or decimal) into DateTime<Utc>.
pub fn parse_timestamp(input: &str, unit: TimestampUnit) -> Result<DateTime<Utc>, String> {
    let trimmed = input.trim();

    // Check if it's a decimal number (e.g. 1718000000.123)
    if trimmed.contains('.') {
        return parse_decimal_timestamp(trimmed, unit);
    }

    let val: i128 = trimmed
        .parse()
        .map_err(|e| format!("Failed to parse integer timestamp '{}': {}", trimmed, e))?;

    let effective_unit = match unit {
        TimestampUnit::Auto => detect_unit(val),
        other => other,
    };

    match effective_unit {
        TimestampUnit::Seconds | TimestampUnit::Auto => {
            if val < (i64::MIN as i128) || val > (i64::MAX as i128) {
                return Err(format!("Timestamp out of range: {}", val));
            }
            DateTime::from_timestamp(val as i64, 0)
                .ok_or_else(|| format!("Invalid timestamp seconds: {}", val))
        }
        TimestampUnit::Milliseconds => {
            let secs = val.div_euclid(1_000);
            let rem_millis = val.rem_euclid(1_000);
            let nsecs = (rem_millis * 1_000_000) as u32;
            if secs < (i64::MIN as i128) || secs > (i64::MAX as i128) {
                return Err(format!("Timestamp out of range: {}", val));
            }
            DateTime::from_timestamp(secs as i64, nsecs)
                .ok_or_else(|| format!("Invalid timestamp milliseconds: {}", val))
        }
        TimestampUnit::Microseconds => {
            let secs = val.div_euclid(1_000_000);
            let rem_micros = val.rem_euclid(1_000_000);
            let nsecs = (rem_micros * 1_000) as u32;
            if secs < (i64::MIN as i128) || secs > (i64::MAX as i128) {
                return Err(format!("Timestamp out of range: {}", val));
            }
            DateTime::from_timestamp(secs as i64, nsecs)
                .ok_or_else(|| format!("Invalid timestamp microseconds: {}", val))
        }
        TimestampUnit::Nanoseconds => {
            let secs = val.div_euclid(1_000_000_000);
            let rem_nanos = val.rem_euclid(1_000_000_000) as u32;
            if secs < (i64::MIN as i128) || secs > (i64::MAX as i128) {
                return Err(format!("Timestamp out of range: {}", val));
            }
            DateTime::from_timestamp(secs as i64, rem_nanos)
                .ok_or_else(|| format!("Invalid timestamp nanoseconds: {}", val))
        }
    }
}

fn parse_decimal_timestamp(trimmed: &str, unit: TimestampUnit) -> Result<DateTime<Utc>, String> {
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid decimal timestamp: {}", trimmed));
    }

    let int_part: i128 = parts[0]
        .parse()
        .map_err(|e| format!("Invalid integer part in timestamp '{}': {}", trimmed, e))?;

    let effective_unit = match unit {
        TimestampUnit::Auto => detect_unit(int_part),
        other => other,
    };

    let frac_str = parts[1];
    let is_negative = int_part < 0 || parts[0].starts_with('-');

    match effective_unit {
        TimestampUnit::Seconds | TimestampUnit::Auto => {
            // Fraction is sub-seconds (nanoseconds has 9 digits)
            let mut nanos_str = frac_str.to_string();
            if nanos_str.len() > 9 {
                nanos_str.truncate(9);
            } else {
                while nanos_str.len() < 9 {
                    nanos_str.push('0');
                }
            }
            let nanos: u32 = nanos_str.parse().unwrap_or(0);
            if is_negative {
                let secs = int_part - 1;
                let adj_nanos = 1_000_000_000 - nanos;
                DateTime::from_timestamp(secs as i64, adj_nanos)
                    .ok_or_else(|| format!("Invalid timestamp: {}", trimmed))
            } else {
                DateTime::from_timestamp(int_part as i64, nanos)
                    .ok_or_else(|| format!("Invalid timestamp: {}", trimmed))
            }
        }
        TimestampUnit::Milliseconds => {
            // Decimal milliseconds: e.g. 1718000000000.5 -> ms + sub-millis
            let mut nanos_str = frac_str.to_string();
            if nanos_str.len() > 6 {
                nanos_str.truncate(6);
            } else {
                while nanos_str.len() < 6 {
                    nanos_str.push('0');
                }
            }
            let sub_ms_nanos: u32 = nanos_str.parse().unwrap_or(0);
            let secs = int_part.div_euclid(1_000);
            let rem_millis = int_part.rem_euclid(1_000);
            let nsecs = (rem_millis * 1_000_000) as u32 + sub_ms_nanos;
            DateTime::from_timestamp(secs as i64, nsecs)
                .ok_or_else(|| format!("Invalid timestamp: {}", trimmed))
        }
        TimestampUnit::Microseconds => {
            let mut nanos_str = frac_str.to_string();
            if nanos_str.len() > 3 {
                nanos_str.truncate(3);
            } else {
                while nanos_str.len() < 3 {
                    nanos_str.push('0');
                }
            }
            let sub_us_nanos: u32 = nanos_str.parse().unwrap_or(0);
            let secs = int_part.div_euclid(1_000_000);
            let rem_micros = int_part.rem_euclid(1_000_000);
            let nsecs = (rem_micros * 1_000) as u32 + sub_us_nanos;
            DateTime::from_timestamp(secs as i64, nsecs)
                .ok_or_else(|| format!("Invalid timestamp: {}", trimmed))
        }
        TimestampUnit::Nanoseconds => {
            let secs = int_part.div_euclid(1_000_000_000);
            let rem_nanos = int_part.rem_euclid(1_000_000_000) as u32;
            DateTime::from_timestamp(secs as i64, rem_nanos)
                .ok_or_else(|| format!("Invalid timestamp: {}", trimmed))
        }
    }
}

/// Convert a DateTime<Utc> to a formatted string according to timezone and format specifier.
pub fn format_datetime(dt: DateTime<Utc>, tz: &TzConfig, format: Option<&str>) -> String {
    let fmt = format.unwrap_or("rfc3339");
    let is_rfc3339 = fmt.eq_ignore_ascii_case("rfc3339")
        || fmt.eq_ignore_ascii_case("rfc-3339")
        || fmt.eq_ignore_ascii_case("iso8601")
        || fmt.eq_ignore_ascii_case("iso-8601");
    let is_rfc2822 = fmt.eq_ignore_ascii_case("rfc2822") || fmt.eq_ignore_ascii_case("rfc-2822");

    fn format_inner<Tz: TimeZone>(
        dt: DateTime<Tz>,
        is_rfc3339: bool,
        is_rfc2822: bool,
        fmt: &str,
        use_z: bool,
    ) -> String
    where
        Tz::Offset: std::fmt::Display,
    {
        if is_rfc3339 {
            dt.to_rfc3339_opts(SecondsFormat::AutoSi, use_z)
        } else if is_rfc2822 {
            dt.to_rfc2822()
        } else {
            dt.format(fmt).to_string()
        }
    }

    match tz {
        TzConfig::Utc => format_inner(dt, is_rfc3339, is_rfc2822, fmt, true),
        TzConfig::Local => format_inner(dt.with_timezone(&Local), is_rfc3339, is_rfc2822, fmt, false),
        TzConfig::Named(named_tz) => {
            format_inner(dt.with_timezone(named_tz), is_rfc3339, is_rfc2822, fmt, false)
        }
        TzConfig::Fixed(offset) => {
            format_inner(dt.with_timezone(offset), is_rfc3339, is_rfc2822, fmt, false)
        }
    }
}

/// Convert a DateTime<Utc> to an epoch timestamp integer in the requested unit.
pub fn datetime_to_timestamp(dt: DateTime<Utc>, unit: TimestampUnit) -> i128 {
    match unit {
        TimestampUnit::Seconds => dt.timestamp() as i128,
        TimestampUnit::Milliseconds | TimestampUnit::Auto => dt.timestamp_millis() as i128,
        TimestampUnit::Microseconds => dt.timestamp_micros() as i128,
        TimestampUnit::Nanoseconds => dt.timestamp_nanos_opt().unwrap_or(0) as i128,
    }
}

/// Parse a date/time string into DateTime<Utc>.
pub fn parse_date(input: &str, tz: &TzConfig, format: Option<&str>) -> Result<DateTime<Utc>, String> {
    let trimmed = input.trim();

    if trimmed.eq_ignore_ascii_case("now") {
        return Ok(Utc::now());
    }

    // If custom format is provided, try that first
    if let Some(fmt) = format {
        if !fmt.eq_ignore_ascii_case("rfc3339")
            && !fmt.eq_ignore_ascii_case("rfc-3339")
            && !fmt.eq_ignore_ascii_case("iso8601")
            && !fmt.eq_ignore_ascii_case("iso-8601")
            && !fmt.eq_ignore_ascii_case("rfc2822")
            && !fmt.eq_ignore_ascii_case("rfc-2822")
        {
            if let Ok(dt) = DateTime::parse_from_str(trimmed, fmt) {
                return Ok(dt.with_timezone(&Utc));
            }
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
                return naive_to_utc(naive_dt, tz);
            }
            if let Ok(naive_date) = NaiveDate::parse_from_str(trimmed, fmt) {
                let naive_dt = naive_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| "Invalid date".to_string())?;
                return naive_to_utc(naive_dt, tz);
            }
        }
    }

    // Try RFC3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try RFC2822
    if let Ok(dt) = DateTime::parse_from_rfc2822(trimmed) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Common ISO / Date / Time format list
    let patterns_with_offset = [
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%dT%H:%M:%S%:z",
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M:%S%:z",
        "%Y-%m-%d %H:%M:%S%z",
    ];

    for pattern in patterns_with_offset {
        if let Ok(dt) = DateTime::parse_from_str(trimmed, pattern) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    let naive_patterns = [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d_%H:%M:%S%.f",
        "%Y-%m-%d_%H:%M:%S",
        "%d.%m.%Y %H:%M:%S%.f",
        "%d.%m.%Y %H:%M:%S",
    ];

    for pattern in naive_patterns {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, pattern) {
            return naive_to_utc(naive_dt, tz);
        }
    }

    let date_only_patterns = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d.%m.%Y",
        "%Y%m%d",
    ];

    for pattern in date_only_patterns {
        if let Ok(naive_date) = NaiveDate::parse_from_str(trimmed, pattern) {
            let naive_dt = naive_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| "Invalid date".to_string())?;
            return naive_to_utc(naive_dt, tz);
        }
    }

    Err(format!(
        "Failed to parse date string '{}'. Expected RFC3339 (e.g. 2024-06-10T06:13:20Z) or YYYY-MM-DD HH:MM:SS",
        trimmed
    ))
}

fn naive_to_utc(naive_dt: NaiveDateTime, tz: &TzConfig) -> Result<DateTime<Utc>, String> {
    match tz {
        TzConfig::Utc => Ok(naive_dt.and_utc()),
        TzConfig::Local => match Local.from_local_datetime(&naive_dt) {
            LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
            LocalResult::Ambiguous(dt1, _) => Ok(dt1.with_timezone(&Utc)),
            LocalResult::None => Err(format!(
                "Local datetime '{}' does not exist due to DST transition",
                naive_dt
            )),
        },
        TzConfig::Named(named_tz) => match named_tz.from_local_datetime(&naive_dt) {
            LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
            LocalResult::Ambiguous(dt1, _) => Ok(dt1.with_timezone(&Utc)),
            LocalResult::None => Err(format!(
                "Datetime '{}' does not exist in timezone '{}' due to DST transition",
                naive_dt,
                named_tz.name()
            )),
        },
        TzConfig::Fixed(offset) => match offset.from_local_datetime(&naive_dt) {
            LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
            LocalResult::Ambiguous(dt1, _) => Ok(dt1.with_timezone(&Utc)),
            LocalResult::None => Err(format!(
                "Datetime '{}' does not exist with offset '{}'",
                naive_dt, offset
            )),
        },
    }
}

/// Helper function to check if an input string looks like a date rather than a numeric timestamp.
pub fn is_date_like(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("now") {
        return true;
    }

    // If it contains characters typical for dates but not for numbers
    if trimmed.contains('-') || trimmed.contains(':') || trimmed.contains('T') || trimmed.contains(' ') || trimmed.contains('/') {
        // Exclude pure negative numbers like -123456789
        if trimmed.starts_with('-') && trimmed[1..].chars().all(|c| c.is_ascii_digit() || c == '.') {
            return false;
        }
        return true;
    }

    // If it contains letters
    if trimmed.chars().any(|c| c.is_alphabetic()) {
        return true;
    }

    false
}

/// Process a single input string according to configuration.
pub fn process_item(
    input: &str,
    reverse: bool,
    unit: TimestampUnit,
    tz: &TzConfig,
    format: Option<&str>,
) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let should_reverse = reverse || is_date_like(trimmed);

    if should_reverse {
        let dt = parse_date(trimmed, tz, format)?;
        let ts = datetime_to_timestamp(dt, unit);
        Ok(ts.to_string())
    } else {
        let dt = parse_timestamp(trimmed, unit)?;
        let out = format_datetime(dt, tz, format);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
