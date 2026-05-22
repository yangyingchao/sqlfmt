use clap::{ArgAction, Parser};
use sqlfmt::config::CaseMode;
use sqlfmt::{format_json, format_sql, FormatterConfig, VERSION};
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(name = "sqlfmt")]
#[command(version = VERSION)]
#[command(about = "SQL formatter with Greenplum support", long_about = None)]
struct Args {
    /// Maximum print width (default: 80)
    #[arg(long, value_name = "WIDTH", default_value = "80")]
    print_width: usize,

    /// Use spaces for indentation instead of tabs
    #[arg(long, action = ArgAction::SetTrue)]
    use_spaces: bool,

    /// Tab width for indentation (default: 4)
    #[arg(long, value_name = "WIDTH", default_value = "4")]
    tab_width: usize,

    /// Keyword case mode (upper, lower, title, spongebob) (default: upper)
    #[arg(long, value_name = "MODE", default_value = "upper")]
    casemode: String,

    /// Do not simplify query structure
    #[arg(long, action = ArgAction::SetTrue)]
    no_simplify: bool,

    /// Align keywords
    #[arg(long, action = ArgAction::SetTrue)]
    align: bool,

    /// Format as JSON
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// SQL statements to format (if not provided, reads from stdin)
    #[arg(long, value_name = "SQL")]
    stmt: Vec<String>,
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
        use_spaces: args.use_spaces,
        tab_width: args.tab_width,
        case_mode,
        simplify: !args.no_simplify,
        align: args.align,
    };

    // Get input SQL
    let input = if !args.stmt.is_empty() {
        args.stmt.join("\n")
    } else {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    };

    if input.trim().is_empty() {
        return Ok(());
    }

    // Format SQL or JSON
    let result = if args.json {
        // Try to format as JSON
        match format_json(&input) {
            Ok(formatted) => formatted,
            Err(_) => {
                // Fall back to treating as SQL if JSON parsing fails
                format_sql(&config, &[input])?
            }
        }
    } else {
        // Format as SQL
        format_sql(&config, &[input])?
    };

    print!("{}", result);
    Ok(())
}
