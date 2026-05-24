# sqlfmt


Forked from: https://github.com/madelynnblue/sqlfmt


Width-aware SQL formatter with Greenplum/PostgreSQL dialect support.

> 本代码由 AI 生成，仅供参考和学习使用。  
> This code is AI-generated and provided for reference and educational purposes only.

Based on Wadler's pretty printer paper:  
http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/sqlfmt --help
```

## Usage

```bash
# Read from stdin, write to stdout
sqlfmt < input.sql

# Read file, write to stdout
sqlfmt input.sql

# Edit file in-place
sqlfmt -i input.sql

# Format with custom width and indent
sqlfmt --print-width 100 --indent-width 2 input.sql

# Use tabs instead of spaces
sqlfmt --tabs input.sql

# Format inline statements
sqlfmt --stmt "SELECT * FROM t WHERE id = 1"
```

### Options

| Flag              | Default      | Description                       |
|-------------------|--------------|-----------------------------------|
| `--print-width`   | `80`         | Maximum line width                |
| `--tabs`          | off (spaces) | Use tabs for indentation          |
| `--indent-width`  | `4`          | Spaces per indent level           |
| `--stmt`          | —            | Inline SQL statement(s) to format |
| `-i`, `--inplace` | off          | Edit file in-place                |
| `-h`, `--help`    | —            | Print help                        |
| `-V`, `--version` | —            | Print version                     |

## Changes from the Go version

This is a Rust reimplementation of the original Go sqlfmt. Key differences:

| Aspect               | Go (original)              | Rust (current)                           |
|----------------------|----------------------------|------------------------------------------|
| Parser               | CockroachDB parser         | sqlparser-rs 0.62                        |
| Width algorithm      | AST-level Wadler printer   | Text-level keyword/comma break detection |
| Indent default       | Tabs                       | 4 spaces                                 |
| Tab flag             | `--use-spaces`             | `--tabs` (inverted)                      |
| Indent width flag    | `--tab-width`              | `--indent-width`                         |
| File arg             | stdin only                 | Supports file path + `-i`/`--inplace`    |
| JSON formatting      | Supported                  | Removed                                  |
| HTTP server          | Supported                  | Removed                                  |
| WASM build           | Supported                  | Removed                                  |
| UTF-8 in SQL         | Supported                  | Supported (fixed char/byte index)        |
| Version sync         | ldflags at build           | `env!("CARGO_PKG_VERSION")`              |

### Width-aware formatting

At default width 80, output is **identical** between the two implementations.

At narrow widths, some AST-level differences exist due to the text-based approach:

- CREATE TABLE column lists are not wrapped independently
- Expression operators like `*` are not broken to separate lines
- `DELETE FROM` is treated as two keywords rather than a phrase

These are inherent limitations of text-level post-processing vs AST-level pretty printing.

## License

MIT
