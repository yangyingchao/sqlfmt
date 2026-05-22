use clap::{ArgAction, Parser};
use sqlfmt::config::CaseMode;
use sqlfmt::{format_sql, FormatterConfig, VERSION};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sqlfmt")]
#[command(version = VERSION)]
#[command(about = "SQL formatter with Greenplum support", long_about = None)]
struct Args {
    /// Maximum print width (default: 80)
    #[arg(long, value_name = "WIDTH", default_value = "80")]
    print_width: usize,

    /// Use tabs for indentation (default: 4 spaces)
    #[arg(long, action = ArgAction::SetTrue)]
    tabs: bool,

    /// Indent width in spaces (default: 4)
    #[arg(long, value_name = "WIDTH", default_value = "4")]
    indent_width: usize,

    /// Keyword case mode (upper, lower, title, spongebob) (default: upper)
    #[arg(long, value_name = "MODE", default_value = "upper")]
    casemode: String,

    /// Do not simplify query structure
    #[arg(long, action = ArgAction::SetTrue)]
    no_simplify: bool,

    /// Align keywords
    #[arg(long, action = ArgAction::SetTrue)]
    align: bool,

    /// SQL statements to format (if not provided, reads from file or stdin)
    #[arg(long, value_name = "SQL")]
    stmt: Vec<String>,

    /// File to format (use - or omit for stdin)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Edit file in-place
    #[arg(short = 'i', long, action = ArgAction::SetTrue)]
    inplace: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse case mode
    let case_mode = match CaseMode::from_str(&args.casemode) {
        Some(mode) => mode,
        None => {
            eprintln!(
                "Invalid case mode: {}. Use: upper, lower, title, spongebob",
                args.casemode
            );
            std::process::exit(1);
        }
    };

    // Create formatter config
    let config = FormatterConfig {
        print_width: args.print_width,
        use_spaces: !args.tabs,
        indent_width: args.indent_width,
        case_mode,
        simplify: !args.no_simplify,
        align: args.align,
    };

    // Get input SQL
    let input = if !args.stmt.is_empty() {
        args.stmt.join("\n")
    } else {
        match &args.file {
            Some(path) if path.as_os_str() != "-" => fs::read_to_string(path)?,
            _ => {
                // Read from stdin
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input
            }
        }
    };

    if input.trim().is_empty() {
        return Ok(());
    }

    let result = format_sql(&config, &[input])?;

    if args.inplace {
        if let Some(path) = &args.file {
            if path.as_os_str() != "-" {
                fs::write(path, &result)?;
                return Ok(());
            }
        }
        eprintln!("error: --inplace requires a file path");
        std::process::exit(1);
    }

    print!("{}", result);
    Ok(())
}
