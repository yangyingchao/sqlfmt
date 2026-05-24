# sqlfmt — agent instructions

## Project

Rust SQL formatter (PostgreSQL/Greenplum), single binary CLI.
Width-aware layout via **text-level** keyword/comma break detection (not AST-level).

**Go version was removed.** Only Rust code remains.

## Entrypoints & structure

| Path                            | Role                                                                 |
|---------------------------------|----------------------------------------------------------------------|
| `src/main.rs`                   | CLI binary (clap)                                                    |
| `src/lib.rs`                    | Library root, re-exports `format_sql`, `FormatterConfig`             |
| `src/formatter/mod.rs`          | Core pipeline orchestrator (8 steps)                                 |
| `src/formatter/pretty.rs`       | Width-aware layout engine                                            |
| `src/formatter/special_clauses.rs` | WITH / DISTRIBUTED BY / PARTITION BY extraction & restoration      |
| `tests/`                        | SQL fixtures + `.out` for comparison, plus `regression_test.rs`      |
| `.github/workflows/release.yml` | CI: `cargo fmt --check`, cross-compile + draft release on tag       |

## Build & run

```bash
cargo build --release
./target/release/sqlfmt input.sql              # file → stdout
./target/release/sqlfmt -i input.sql           # in-place edit
./target/release/sqlfmt < input.sql            # stdin → stdout
./target/release/sqlfmt --tabs                 # use tabs (default: 4 spaces)
sqlfmt --stmt "SELECT 1" --stmt "SELECT 2"    # inline stmts (joined by newline)
```

## Verification

Rust unit tests (`cargo test`) plus idempotency check:

```bash
cargo test
cargo build --release
# idempotency check (format twice, output should be identical):
./target/release/sqlfmt tests/test-distributed-by.sql | ./target/release/sqlfmt | \
  diff tests/test-distributed-by.out -
```

CI also runs `cargo fmt --check` and cross-compiles for linux-musl.

## Key CLI details

- `--tabs` inverted flag: default is spaces, `--tabs` enables tabs
- `--indent-width` controls space-indent count AND tab-width-in-chars calculation
- `-i` / `--inplace` requires a file path (errors otherwise)
- `--stmt` values joined with `\n`, not `;` — sqlparser fails on multi-stmt newline joins

## Width-formatter limitations (text-level, not AST-level)

- CREATE TABLE columns not wrapped independently
- Expression operators (`*`) not broken to separate lines
- `DELETE FROM` treated as two keywords, not a phrase
- At default width 80, output matches original Go version exactly

## Conventions

- **Do NOT auto-commit** — wait for explicit user request
- **Before committing**, ensure `cargo test` passes with no failures.
- `VERSION` auto-syncs from `Cargo.toml` via `env!("CARGO_PKG_VERSION")`
- If you edit rust source files, makes sure to run `cargo fmt` after your edit.
- Ensure `cargo fmt -- --check` does not report warnings or errors.
- Formatting pipeline order: split → extract comments → extract special clauses → parse → restore clauses → keyword normalization (uppercase only, Greenplum-specific) → width formatting → semicolon → leading comments
- Uses `PostgreSqlDialect` (not `GenericDialect`) — TEXT type preserved natively, no TEXT/STRING swap needed
