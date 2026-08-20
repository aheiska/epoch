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
- **Timezone Conversion**: Supports `UTC`, system `local`, fixed offsets (e.g., `+02:00`, `-05:00`), and named IANA timezones (e.g., `Europe/Helsinki`, `America/New_York`).
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
  -z, --tz <TIMEZONE>    Target timezone (e.g. 'UTC', 'local', 'Europe/Helsinki', '+02:00', '-05:00') [default: UTC]
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
- **Avoid Seasonal Abbreviations**: 3–4 letter abbreviations such as `EEST`, `EDT`, `CEST`, or `BST` are informal seasonal labels and are not distinct timezone identifiers. Using `Europe/Helsinki` will automatically apply `+03:00` (EEST) in the summer and `+02:00` (EET) in the winter.
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
