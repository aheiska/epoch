use clap::Parser;
use epoch::{process_item, TimestampUnit, TzConfig};
use std::io::{self, BufRead};
use std::process;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "epoch",
    version,
    about = "CLI tool to convert timestamps (ms, s, us, ns) to RFC3339/custom dates and vice versa.",
    after_help = "EXAMPLES:\n  \
      epoch 1718000000000\n  \
      echo 1718000000000 | epoch\n  \
      epoch -z Europe/Helsinki 1718000000000\n  \
      epoch -f \"%Y-%m-%d %H:%M:%S\" 1718000000000\n  \
      epoch 2024-06-10T06:13:20Z\n  \
      epoch -u s 2024-06-10T06:13:20Z\n  \
      epoch -u us 1718000000000000"
)]
struct Cli {
    /// Inputs to convert (timestamps or date strings). If empty or '-', reads from stdin line-by-line.
    #[arg(value_name = "INPUT")]
    inputs: Vec<String>,

    /// Target timezone (e.g. 'UTC', 'local', 'Europe/Helsinki', '+02:00', '-05:00').
    #[arg(short = 'z', long = "tz", visible_alias = "timezone", default_value = "UTC")]
    timezone: String,

    /// Output/input format string (e.g. '%Y-%m-%d %H:%M:%S', 'rfc3339', 'rfc2822').
    #[arg(short = 'f', long = "format")]
    format: Option<String>,

    /// Timestamp unit: auto, s (seconds), ms (milliseconds), us (microseconds), ns (nanoseconds).
    #[arg(short = 'u', long = "unit", default_value = "auto")]
    unit: String,

    /// Force conversion from date to timestamp (reverse mode). Automatically inferred for date-like strings.
    #[arg(short = 'r', long = "reverse", visible_alias = "from-date")]
    reverse: bool,
}

fn parse_unit(unit_str: &str) -> TimestampUnit {
    TimestampUnit::from_str(unit_str).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    })
}

fn parse_timezone(tz_str: &str) -> TzConfig {
    TzConfig::from_str(tz_str).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    })
}

fn is_stdin_requested(inputs: &[String]) -> bool {
    inputs.is_empty() || (inputs.len() == 1 && inputs[0] == "-")
}

fn process_and_print_item(
    item: &str,
    reverse: bool,
    unit: TimestampUnit,
    tz: &TzConfig,
    format: Option<&str>,
) -> bool {
    let trimmed = item.trim();
    if trimmed.is_empty() {
        return true;
    }

    match process_item(trimmed, reverse, unit, tz, format) {
        Ok(out) => {
            if !out.is_empty() {
                println!("{}", out);
            }
            true
        }
        Err(e) => {
            eprintln!("error: {}", e);
            false
        }
    }
}

fn process_inputs<'a, I>(
    inputs: I,
    reverse: bool,
    unit: TimestampUnit,
    tz: &TzConfig,
    format: Option<&str>,
) -> bool
where
    I: IntoIterator<Item = &'a str>,
{
    inputs
        .into_iter()
        .fold(true, |success, item| {
            let ok = process_and_print_item(item, reverse, unit, tz, format);
            success && ok
        })
}

fn process_stdin(
    reverse: bool,
    unit: TimestampUnit,
    tz: &TzConfig,
    format: Option<&str>,
) -> bool {
    let stdin = io::stdin();
    let mut success = true;

    for line_res in stdin.lock().lines() {
        match line_res {
            Ok(line) => {
                let ok = process_and_print_item(&line, reverse, unit, tz, format);
                success = success && ok;
            }
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                return false;
            }
        }
    }

    success
}

fn run(cli: Cli) -> Result<(), ()> {
    let unit = parse_unit(&cli.unit);
    let tz = parse_timezone(&cli.timezone);
    let format = cli.format.as_deref();

    let success = if is_stdin_requested(&cli.inputs) {
        process_stdin(cli.reverse, unit, &tz, format)
    } else {
        process_inputs(
            cli.inputs.iter().map(String::as_str),
            cli.reverse,
            unit,
            &tz,
            format,
        )
    };

    if success {
        Ok(())
    } else {
        Err(())
    }
}

fn main() {
    let cli = Cli::parse();
    if run(cli).is_err() {
        process::exit(1);
    }
}
