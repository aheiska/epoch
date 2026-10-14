# epoch

A fast, lightweight CLI utility to convert timestamps (seconds, milliseconds, microseconds, nanoseconds) to human-readable RFC3339/custom date strings and vice versa.

## Motivation

Working with timestamps in the Linux command line is often a frustrating experience, 
especially when dealing with millisecond or microsecond timestamps from logs, 
databases, and APIs. Standard CLI utilities like `date` or `awk` typically 
require manual arithmetic (`/ 1000`) or other unnecessary complexity.

`epoch` makes epoch timestamp conversion frictionless:
- Seamless stdin piping and command-line argument support.
- Automatic unit detection (seconds, ms, µs, ns).
- Default to UTC, but support timezones with automatic Daylight Saving Time (DST) handling.
- Flexible custom date formatting.
- Bidirectional conversion (dates to timestamps and timestamps to dates).

---

## Features

- **Auto-detection**: Automatically determines whether an integer timestamp is in seconds, milliseconds, microseconds, or nanoseconds based on magnitude.
- **Bidirectional**: Converts timestamps to dates and formatted dates back to epoch timestamps. Tries hard to detect the correct function from the input format
- **Streaming & Batching**: Accepts input via CLI arguments or streamed line-by-line through `stdin`.
- **Timezone Conversion**: Supports `UTC`, system `local`, fixed offsets (e.g., `+02:00`, `-05:00`), supported unambiguous abbreviations (e.g., `EEST`), and named IANA timezones (e.g., `Europe/Helsinki`, `America/New_York`).
- **Custom Formatting**: Supports standard `strftime` formats as well as presets like `rfc3339` and `rfc2822`.
- **Decimal / Sub-second Precision**: Supports fractional timestamps (e.g., `1718000000.123`).

---

## Installation

### From Source

```bash
cargo install --path .
```

Or build a release binary:

```bash
cargo build --release
```

---

## Usage & Examples

```
Usage: epoch [OPTIONS] [INPUT]...

Arguments:
  [INPUT]...  Inputs to convert (timestamps or date strings). If empty or '-', reads from stdin line-by-line.

Options:
  -z, --tz <TIMEZONE>    Target timezone (e.g. 'UTC', 'local', 'Europe/Helsinki', 'EEST', '+02:00'). Abbreviations use fixed offsets, without DST changes. [default: UTC]
  -f, --format <FORMAT>  Output/input format string (e.g. '%Y-%m-%d %H:%M:%S', 'rfc3339', 'rfc2822')
  -u, --unit <UNIT>      Timestamp unit: auto, s, ms, us, ns [default: auto]
  -r, --reverse          Force conversion from date to timestamp (reverse mode)
  -h, --help             Print help
  -V, --version          Print version
```

### Basic Conversions

```bash
# Auto-detects milliseconds -> outputs RFC3339 in UTC
$ epoch 1718000000000
2024-06-10T06:13:20Z

# Converts seconds with decimal sub-seconds
$ epoch 1718000000.500
2024-06-10T06:13:20.500Z

# Auto detects date strings -> outputs milliseconds
$ epoch 2024-06-10
1717977600000

# Multiple arguments
$ epoch 1718000000 1718000001
2024-06-10T06:13:20Z
2024-06-10T06:13:21Z

# Can mix date strings and timestamps
$ epoch "2024-06-10 00:00:00" 2024-06-11 2024-06-10T06:13:20Z 1718064000000
1717977600000
1718064000000
1718000000000
2024-06-11T00:00:00Z
```

### Reduced-precision Date Inputs

For `YYYY-MM-DD` dates followed by `T` or a space, trailing time components may be omitted:

- `2026-06-01T12` means `2026-06-01T12:00:00`.
- `2026-06-01T12:30` means `2026-06-01T12:30:00`.
- `2026-06-01 12` and `2026-06-01 12:30` work the same way.

Only missing trailing components default to zero; invalid supplied values are rejected.
These forms have no embedded timezone: they use `--tz` (UTC by default), including
the usual daylight-saving rules for named zones and fixed offsets for abbreviations.
Include seconds when supplying an embedded timezone offset or `Z` (e.g. `2026-06-01T12:00:00Z`).

```bash
$ epoch '2026-06-01T12'
1780315200000
```

### Working with `stdin` Streams

Pipe logs, JSON streams, or extracted columns directly:

```bash
# From echo or pipes
echo 1718000000000 | epoch

# Parsing timestamp columns from log files
cat access.log | awk '{print $1}' | epoch -z local
```

### Timezones

```bash
# Convert to a specific IANA timezone (handles DST automatically)
epoch -z Europe/Helsinki 1718000000000
# Output: 2024-06-10T09:13:20+03:00

# Convert using system local timezone
epoch -z local 1718000000000

# Convert using fixed offset
epoch -z +02:00 1718000000000
```

#### Timezone Abbreviations

`-z` / `--tz` accepts the following explicit set of unambiguous abbreviations,
case-insensitively. Each means **one constant UTC offset**, not a geographic
timezone. No automatic timezone, daylight-saving, seasonal, or historical offset
change is made with abbreviations, even if the abbreviation contains "Summer" or
"Daylight". Abbreviations outside this table are not supported as abbreviation
aliases; IANA timezone identifiers remain supported except for the explicitly
rejected ambiguous names listed below.

| Abbreviation | Meaning | Fixed UTC offset |
| --- | --- | --- |
| `UTC`, `Z` | Coordinated Universal Time | `+00:00` |
| `GMT` | Greenwich Mean Time | `+00:00` |
| `WET` | Western European Time | `+00:00` |
| `WEST` | Western European Summer Time | `+01:00` |
| `CET` | Central European Time | `+01:00` |
| `CEST` | Central European Summer Time | `+02:00` |
| `EET` | Eastern European Time | `+02:00` |
| `EEST` | Eastern European Summer Time | `+03:00` |
| `EDT` | Eastern Daylight Time | `-04:00` |
| `MDT` | Mountain Daylight Time | `-06:00` |
| `PDT` | Pacific Daylight Time | `-07:00` |
| `AKST` | Alaska Standard Time | `-09:00` |
| `AKDT` | Alaska Daylight Time | `-08:00` |
| `HST` | Hawaii Standard Time | `-10:00` |
| `HKT` | Hong Kong Time | `+08:00` |
| `JST` | Japan Standard Time | `+09:00` |
| `KST` | Korea Standard Time | `+09:00` |
| `NPT` | Nepal Time | `+05:45` |
| `AWST` | Australian Western Standard Time | `+08:00` |
| `AEST` | Australian Eastern Standard Time | `+10:00` |
| `AEDT` | Australian Eastern Daylight Time | `+11:00` |
| `ACDT` | Australian Central Daylight Time | `+10:30` |
| `NZST` | New Zealand Standard Time | `+12:00` |
| `NZDT` | New Zealand Daylight Time | `+13:00` |
| `CHAST` | Chatham Standard Time | `+12:45` |
| `CHADT` | Chatham Daylight Time | `+13:45` |

For example, midnight UTC on January 1, 2026 renders as **03:00 with `EEST`,
not 02:00**, although Helsinki observes EET in January:

```bash
epoch -z EEST 1767225600
# Output: 2026-01-01T03:00:00+03:00

epoch -z Europe/Helsinki 1767225600
# Output: 2026-01-01T02:00:00+02:00

# Reverse conversion: an offset-free date is interpreted at the fixed +03:00
epoch -z EEST -u s "2026-01-01T03:00:00"
# Output: 1767225600
```

An offset-free input `2026-01-01T00:00:00` interpreted **in EEST** means
`2025-12-31T21:00:00Z`. Date inputs that already contain `Z` or a numeric offset
keep their explicit offset; `--tz` does not reinterpret them. Fixed-offset aliases
(other than `UTC` / `Z`) use numeric offsets in output, including custom `%Z`
formatting; the original abbreviation label is not retained. To apply actual seasonal rules, use a
geographic IANA name instead.

#### Unsupported Ambiguous Abbreviations

These names are rejected because the same abbreviation names multiple timezones
with different offsets. Examples include historical uses: a name is not made
unique merely because one of its meanings is no longer in use. This also rejects
legacy IANA identifiers such as `EST` and `MST` rather than silently selecting
one meaning.

| Rejected abbreviation | Conflicting meanings (UTC offsets) |
| --- | --- |
| `CST` | Central Standard (`-06:00`), China Standard (`+08:00`), Cuba Standard (`-05:00`) |
| `CDT` | Central Daylight (`-05:00`), Cuba Daylight (`-04:00`) |
| `IST` | India Standard (`+05:30`), Irish Standard (`+01:00`), Israel Standard (`+02:00`) |
| `BST` | British Summer (`+01:00`), Bangladesh Standard (`+06:00`), Bougainville Standard (`+11:00`) |
| `EST` | Eastern Standard in North America (`-05:00`), Eastern Standard in Australia (`+10:00`) |
| `MST` | Mountain Standard (`-07:00`), historical Malaya Standard (`+07:30`) |
| `PST` | Pacific Standard (`-08:00`), Philippine Standard (`+08:00`) |
| `AST` | Atlantic Standard (`-04:00`), Arabia Standard (`+03:00`) |
| `ADT` | Atlantic Daylight (`-03:00`), historical Arabia Daylight (`+04:00`) |
| `GST` | Gulf Standard (`+04:00`), South Georgia Time (`-02:00`) |
| `SST` | Samoa Standard (`-11:00`), Singapore Standard (`+08:00`) |
| `NST` | Newfoundland Standard (`-03:30`), historical North Sumatra Time (`+06:30`) |
| `AMT` | Armenia Time (`+04:00`), Amazon Time (`-04:00`) |
| `WST` | Western Standard (`+08:00`), Western Samoa Time (`+13:00`) |
| `ACST` | Australian Central Standard (`+09:30`), historical Acre Summer (`-04:00`) |

Use a geographic name such as `America/Chicago`, `Asia/Shanghai`,
`Asia/Kolkata`, or `Australia/Darwin` to specify a region's rules, or a numeric
offset such as `-06:00`, `+05:30`, or `+09:30` to request a constant offset.

### Custom Formats

```bash
# Custom strftime format
epoch -f "%Y-%m-%d %H:%M:%S" 1718000000000
# Output: 2024-06-10 06:13:20

# RFC 2822 email date format
epoch -f rfc2822 1718000000
# Output: Mon, 10 Jun 2024 06:13:20 +0000
```

### Reverse Conversion (Date to Timestamp)

`epoch` automatically detects date strings or can be explicitly forced with `-r` / `--reverse`:

```bash
# Convert RFC3339 date to millisecond timestamp (default unit: ms)
epoch 2024-06-10T06:13:20Z
# Output: 1718000000000

# Convert date to second timestamp
epoch -u s 2024-06-10T06:13:20Z
# Output: 1718000000

# Convert date to microsecond timestamp
epoch -u us "2024-06-10 06:13:20" -f "%Y-%m-%d %H:%M:%S"
# Output: 1718000000000000
```

---

## Finding Valid IANA Timezone Names

`epoch` uses standard **IANA Time Zone Database** (`tzdb`) identifiers (via `chrono-tz`).

### Best Practices for Timezones
- **Use Geographic Names**: Always prefer geographic area/city identifiers such as `Europe/Helsinki`, `America/New_York`, `Asia/Tokyo`, or `UTC`. Geographic identifiers contain full historical and future transition rules for Daylight Saving Time (DST).
- **Abbreviations Are Fixed Offsets**: Supported abbreviations such as `EEST`, `EDT`, and `CEST` never adjust for the date or DST; ambiguous abbreviations such as `BST` are rejected. Use `Europe/Helsinki` to automatically apply `+03:00` (EEST) in the summer and `+02:00` (EET) in the winter. See the supported and unsupported abbreviation tables above.
- **Fixed Offsets**: If you need an exact static offset without DST transitions, use fixed offset format: `+02:00`, `-05:00`, `+0530`.

### How to Find Timezone Names on Your System

- **Linux (`systemd`)**:
  ```bash
  timedatectl list-timezones
  # Filter by region:
  timedatectl list-timezones | grep -i Helsinki
  ```

- **Linux / macOS file system**:
  ```bash
  # Check available zone files
  ls /usr/share/zoneinfo/
  ls /usr/share/zoneinfo/Europe/
  ```

- **Online Reference**:
  Browse the [Wikipedia: List of tz database time zones](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).
