use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use esquery_rs::{JsSourceType, MatchResult};

const CLI_VERSION: &str = match option_env!("EG_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Parser)]
#[command(
    name = "eg",
    about = "grep JS/TS files with ESQuery selectors",
    version = CLI_VERSION
)]
struct Cli {
    /// File glob pattern (e.g., "src/**/*.ts")
    pattern: String,

    /// ESQuery selector (e.g., "Identifier", "BinaryExpression[operator=\"+\"]")
    selector: String,

    /// Force source type instead of inferring from extension
    #[arg(short = 't', long = "type", value_parser = parse_source_type)]
    source_type: Option<JsSourceType>,
}

fn parse_source_type(s: &str) -> Result<JsSourceType, String> {
    match s {
        "js" => Ok(JsSourceType::Js),
        "jsx" => Ok(JsSourceType::Jsx),
        "ts" => Ok(JsSourceType::Ts),
        "tsx" => Ok(JsSourceType::Tsx),
        _ => Err(format!(
            "unknown source type: {s} (expected: js, jsx, ts, tsx)"
        )),
    }
}

fn infer_source_type(path: &Path) -> Option<JsSourceType> {
    match path.extension()?.to_str()? {
        "js" | "mjs" | "cjs" => Some(JsSourceType::Js),
        "jsx" => Some(JsSourceType::Jsx),
        "ts" | "mts" | "cts" => Some(JsSourceType::Ts),
        "tsx" => Some(JsSourceType::Tsx),
        _ => None,
    }
}

/// Convert a UTF-8 byte offset to (line, column), both 1-based.
fn byte_offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let offset = (offset as usize).min(source.len());
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn print_match(path: &str, source: &str, m: &MatchResult) {
    let (line, col) = byte_offset_to_line_col(source, m.start);
    // Show first line of matched text only
    let display_text = m.text.lines().next().unwrap_or("");
    println!("{path}:{line}:{col}: {display_text}");
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut found = false;

    let paths = match glob::glob(&cli.pattern) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("invalid glob pattern: {e}");
            return ExitCode::from(2);
        }
    };

    for entry in paths {
        let path = match entry {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let source_type = cli.source_type.or_else(|| infer_source_type(&path));
        let Some(source_type) = source_type else {
            continue; // skip files with unknown extensions
        };

        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                continue;
            }
        };

        let path_str = path.to_string_lossy();
        let results = esquery_rs::query(&source, &cli.selector, source_type);
        for m in &results {
            print_match(&path_str, &source, m);
            found = true;
        }
    }

    if found {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
