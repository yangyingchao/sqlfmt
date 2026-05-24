use clap::{ArgAction, Parser};
use sqlfmt::{format_sql, FormatterConfig, VERSION};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
enum InputEncoding {
    Utf8,
    Gb18030,
}

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

/// Read SQL file, detect encoding, return decoded text + detected encoding.
fn read_sql_file(path: &Path) -> io::Result<(String, InputEncoding)> {
    let bytes = fs::read(path)?;
    if let Ok(s) = std::str::from_utf8(&bytes) {
        return Ok((s.to_owned(), InputEncoding::Utf8));
    }
    let (decoded, _, had_errors) = encoding_rs::GB18030.decode(&bytes);
    if had_errors {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: not valid UTF-8 or GB18030", path.display()),
        ));
    }
    Ok((decoded.into_owned(), InputEncoding::Gb18030))
}

fn encode_output(text: &str, encoding: InputEncoding) -> Vec<u8> {
    match encoding {
        InputEncoding::Utf8 => text.as_bytes().to_vec(),
        InputEncoding::Gb18030 => {
            let (encoded, _, _) = encoding_rs::GB18030.encode(text);
            encoded.into_owned()
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create formatter config
    let config = FormatterConfig {
        print_width: args.print_width,
        use_spaces: !args.tabs,
        indent_width: args.indent_width,
    };

    // Get input SQL and detect encoding
    let (input, encoding) = if !args.stmt.is_empty() {
        (args.stmt.join("\n"), InputEncoding::Utf8)
    } else {
        match &args.file {
            Some(path) if path.as_os_str() != "-" => read_sql_file(path)?,
            _ => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                (input, InputEncoding::Utf8)
            }
        }
    };

    if input.trim().is_empty() {
        return Ok(());
    }

    let result = format_sql(&config, &[input])?;
    let encoded = encode_output(&result, encoding);

    if args.inplace {
        if let Some(path) = &args.file {
            if path.as_os_str() != "-" {
                fs::write(path, &encoded)?;
                return Ok(());
            }
        }
        eprintln!("error: --inplace requires a file path");
        std::process::exit(1);
    }

    io::stdout().write_all(&encoded)?;
    Ok(())
}
