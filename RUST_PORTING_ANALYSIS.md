# sqlfmt Go to Rust Porting Analysis

**Project**: sqlfmt - SQL formatter based on Wadler's pretty printer algorithm
**Original Language**: Go
**Target Language**: Rust
**Analysis Date**: May 22, 2026

---

## 1. CORE LIBRARY ANALYSIS (sqlfmt.go)

### 1.1 Public API Functions

#### `FmtSQL(cfg tree.PrettyCfg, stmts []string) -> (string, error)`
- **Purpose**: Main entry point for SQL formatting
- **Input**: Configuration object and list of SQL statements
- **Output**: Formatted SQL string or error
- **Flow**:
  1. Iterate through each statement
  2. Strip TEXT types (CockroachDB converts TEXT to STRING)
  3. Extract special clauses (WITH, DISTRIBUTED BY, PARTITION BY)
  4. Process comments and split statements by semicolons
  5. Parse each statement
  6. Apply pretty printing with configuration
  7. Restore TEXT types and special clauses
  8. Return formatted result

#### `FmtJSON(s string) -> (pretty.Doc, error)`
- **Purpose**: Format JSON input as pretty-printed document
- **Input**: JSON string
- **Output**: Pretty document or error
- **Implementation**: Recursive tree traversal via `fmtJSONNode()`

### 1.2 Regular Expression Patterns

| Pattern Name | Regex | Purpose |
|---|---|---|
| `ignoreComments` | `^--.*\s*` | Match SQL line comments for preservation |
| `distributedByPattern` | `(?i)\s+distributed\s+by\s*\([^)]*\)` | Extract DISTRIBUTED BY clause |
| `partitionByPattern` | `(?i)\s+partition\s+by\s+\w+\s*\([^)]*\)` | Extract PARTITION BY clause |
| `withClausePattern` | `(?i)\s+with\s*\([^)]*\)` | Extract WITH clause |
| `textTypePattern` | `(?i)\bTEXT\b` | Locate TEXT keyword occurrences |

### 1.3 TEXT Type Restoration Logic

**Problem**: CockroachDB parser converts `TEXT` to `STRING` during parsing.

**Solution - Two-Phase Approach**:
1. **stripTextType()**: Count TEXT occurrences before parsing (non-destructive)
2. **restoreTextType()**: Replace first N STRING occurrences with TEXT after formatting
   - Maintains count of replaced instances
   - Preserves case from match

**Limitations**: This approach assumes TEXT→STRING 1:1 mapping; edge cases with pre-existing STRING types may cause incorrect restoration.

### 1.4 Special Clause Extraction/Restoration Flow

#### Extraction Phase - `extractAndStripClauses()`

**Input**: SQL string
**Output**: Cleaned SQL + Map of clauses per CREATE TABLE statement

**Process**:
1. Find all CREATE TABLE statements using regex
2. For each CREATE TABLE:
   - Extract WITH clause (both original multi-line and compressed single-line)
   - Extract DISTRIBUTED BY clause
   - Extract PARTITION BY clause
   - Store in map with keys: "WITH", "WITH_ORIGINAL", "DISTRIBUTED", "PARTITION"
3. Strip all extracted clauses from SQL
4. Return cleaned SQL and list of clause maps

**Data Structure**:
```go
allClauses []map[string][]string
// Example:
// {
//   "WITH": ["WITH ( APPENDONLY = true, ... )"],
//   "WITH_ORIGINAL": ["with (\n    appendonly = true,\n    ...\n)"],
//   "DISTRIBUTED": ["DISTRIBUTED BY (a)"],
//   "PARTITION": ["PARTITION BY list(a)"],
// }
```

#### Restoration Phase - `restoreAllSpecialClauses()`

**Inputs**: 
- Formatted SQL string
- Clause maps from extraction
- lineWidth for intelligent compression decision

**Process**:
1. Find CREATE TABLE statements in formatted SQL
2. For each match, retrieve corresponding clause set
3. **Intelligent WITH compression**:
   - Check if compressed version fits within lineWidth
   - If YES: use compressed (single-line) version
   - If NO: use original (multi-line) version
4. Append clauses in order: WITH → DISTRIBUTED BY → PARTITION BY
5. Normalize keywords to uppercase before appending
6. Return SQL with restored clauses

**Example**:
```
Original (79 chars, fits in 80-width):
WITH ( APPENDONLY = true, COMPRESSLEVEL = 3 )

Original (multi-line, doesn't fit):
WITH (
    APPENDONLY = true,
    ORIENTATION = column,
    COMPRESSTYPE = multiple,
    COMPRESSLEVEL = 3
)
```

### 1.5 WITH Clause Compression Algorithm - `mergeLineIfFits()`

**Purpose**: Intelligently decide whether to compress multi-line WITH clauses

**Logic**:
```
merged_length = len(prevLine) + 1 + len(currentLine)
if merged_length <= lineWidth:
    return merged_line
else:
    return original (two-line)
```

### 1.6 Keyword Normalization Functions

| Function | Input | Output | Purpose |
|---|---|---|---|
| `normalizeWithClause()` | `with(...)` | `WITH(...)` | Uppercase WITH keyword, normalize parameter names |
| `normalizeDistributedByKeyword()` | `distributed by` | `DISTRIBUTED BY` | Uppercase both words |
| `normalizePartitionByKeyword()` | `partition by` | `PARTITION BY` | Uppercase both words |

### 1.7 Data Structures

#### Key Types
```go
// Simple types
string            // SQL text, clause strings
[]string          // List of statements, clause values
int               // Counts, configuration values

// Aggregates
map[string][]string  // Clause storage: {key: [value]}
[]map[string][]string // Multi-statement clause sets

// External (tree package)
tree.PrettyCfg       // Configuration object
tree.ParsedStatement  // Parsed AST
pretty.Doc           // Pretty-printed document
json.JSON            // JSON AST node
```

### 1.8 Comment Handling

- Comments are preserved using `ignoreComments` regex
- Trailing whitespace stripped from comments
- Newline count capped at 2 to prevent excessive blank lines
- Comments processed before statement parsing

---

## 2. CLI & HTTP SERVER ANALYSIS (backend/main.go)

### 2.1 Command-Line Flags

| Flag | Type | Default | Purpose |
|---|---|---|---|
| `--print-width` | int | 80 | Line length where sqlfmt will wrap |
| `--use-spaces` | bool | false | Indent with spaces instead of tabs |
| `--tab-width` | int | 4 | Number of spaces per indentation level |
| `--casemode` | string | "upper" | Keyword casing (upper\|lower\|title\|spongebob) |
| `--no-simplify` | bool | false | Don't simplify output (remove unneeded parens) |
| `--align` | bool | false | Right-align keywords |
| `--stmt` | []string | nil | SQL statements as arguments (instead of stdin) |
| `--help` | bool | false | Display help (-h alias) |
| `--version` | bool | false | Display version (-v alias) |

### 2.2 Flag to Configuration Mapping

```go
cfg := tree.DefaultPrettyCfg()
cfg.UseTabs = !*flagUseSpaces           // Inverted logic
cfg.LineWidth = *flagPrintWidth
cfg.TabWidth = *flagTabWidth
cfg.Simplify = !*flagNoSimplify         // Inverted logic
cfg.Align = tree.PrettyNoAlign           // Default
if *flagAlign {
    cfg.Align = tree.PrettyAlignAndDeindent
}
cfg.Case = caseModes[*flagCasemode]     // Function mapping
cfg.JSONFmt = true                      // Always enabled
```

### 2.3 Input Handling

**CLI Mode**:
1. Check if `--stmt` arguments provided
2. If YES: use those as statements
3. If NO: read all of stdin until EOF
4. Combine into single string (one statement string per input)

**Code**:
```go
sl := *flagStmts  // Get array of statements
if len(sl) == 0 {
    in, err := ioutil.ReadAll(os.Stdin)
    sl = append(sl, string(in))  // Wrap stdin in slice
}
```

### 2.4 Mode Selection Logic

```
SQLFMT_ADDR environment variable set?
├─ YES → HTTP Server mode
└─ NO  → CLI mode
```

**Note**: Same binary, different execution paths based on environment.

### 2.5 HTTP Server Features

#### Endpoints
- `/` - Index page with interactive editor
- `/about` - About page
- `/editor` - Editor configuration page
- `/fmt` - API endpoint for formatting

#### `/fmt` Query Parameters

| Parameter | Type | Default | Mapping |
|---|---|---|---|
| `sql` | string | required | SQL input |
| `n` | int | required | LineWidth |
| `indent` | int | required | TabWidth |
| `simplify` | string | on\|off | Simplify boolean |
| `spaces` | string | on\|off | UseTabs (inverted) |
| `align` | int | required | Align mode (0-3) |
| `case` | string | required | Case transformation |
| `json` | string | optional | Response format |

#### Response Format

**Without `json` parameter** (text/plain):
```
<formatted_sql>
```

**With `json=1` parameter** (application/json):
```json
{
  "Data": "<formatted_sql_or_error>",
  "Error": false
}
```

#### Special Parsing Logic

```go
func parseBool(val string) bool {
    switch val {
    case "on":   return true
    case "off":  return false
    default:     return strconv.ParseBool(val)
    }
}
```

### 2.6 Caching

**Type**: In-memory LRU cache
**Key**: Raw query string (`r.URL.RawQuery`)
**Max size**: 10,000 entries
**Eviction**: When size > 10k, clear entire map
**Thread safety**: Protected by sync.RWMutex

### 2.7 JSON Fallback

If SQL parsing fails, server attempts JSON formatting:
```go
if err == nil {
    return res  // SQL success
}
if jsonDoc, jErr := sqlfmt.FmtJSON(sql); jErr == nil && jsonDoc != nil {
    return pretty.Pretty(jsonDoc, ...)  // JSON fallback
}
return err  // Both failed
```

### 2.8 Case Mode Implementations

| Mode | Function | Behavior |
|---|---|---|
| "upper" | `strings.ToUpper` | ALL CAPS |
| "lower" | `strings.ToLower` | all lowercase |
| "title" | `titleCase` | Title Case (applies `strings.Title` after lowercasing) |
| "spongebob" | `spongeBobCase` | RaNdOm CaSe (uses `rand.Intn(2)` per character) |

```go
func titleCase(s string) string {
    return strings.Title(strings.ToLower(s))
}

func spongeBobCase(s string) string {
    var b strings.Builder
    b.Grow(len(s))
    for _, c := range s {
        b.WriteRune(unicode.To(rand.Intn(2), c))
    }
    return b.String()
}
```

### 2.9 Validation

**CLI validation** (runCmd):
- `--print-width` must be > 0
- `--tab-width` must be > 0
- `--casemode` must exist in caseModes map

**HTTP validation** (fmtSQLRequest):
- `n` parameter must be valid int
- `indent` parameter must be valid int
- `simplify` parameter must be parseable bool
- `align` parameter must be valid int
- `case` parameter must exist in caseModes map
- `spaces` parameter must be parseable bool

---

## 3. DEPENDENCIES ANALYSIS

### 3.1 Go Imports

#### Direct (Required)

| Import | Package | Purpose |
|---|---|---|
| `regexp` | stdlib | Regular expression matching |
| `strings` | stdlib | String manipulation |
| `unicode` | stdlib | Character classification |
| `encoding/json` | stdlib | JSON encoding (Go's json, not CockroachDB's) |
| `fmt` | stdlib | Formatted I/O |
| `html/template` | stdlib | Template rendering for HTTP |
| `io/ioutil` | stdlib | I/O utilities (deprecated in Go 1.16+) |
| `log` | stdlib | Logging |
| `math/rand` | stdlib | Random number generation |
| `net/http` | stdlib | HTTP server/client |
| `os` | stdlib | OS operations (stdin, env, signals) |
| `os/signal` | stdlib | Signal handling |
| `strconv` | stdlib | String conversion |
| `sync` | stdlib | Synchronization primitives |
| `syscall` | stdlib | System calls |
| `github.com/cockroachdb/cockroachdb-parser` | v0.25.2 | SQL parser + tree structures |
| `github.com/kelseyhightower/envconfig` | v1.4.0 | Environment config parsing |
| `github.com/spf13/pflag` | v1.0.10 | Flag parsing (POSIX-style) |
| `golang.org/x/crypto` | v0.43.0 | ACME/TLS support |

### 3.2 CockroachDB Parser API Usage

**From `github.com/cockroachdb/cockroachdb-parser`**:

#### Subpackage: `pkg/sql/parser`
```go
parser.SplitFirstStatement(sql string) (int, error)
// Splits SQL into statements, returns position of first statement end
// Used to properly handle semicolon-separated statements

parser.Parse(sql string) ([]ParsedStatement, error)
// Parses SQL and returns AST nodes
```

#### Subpackage: `pkg/sql/sem/tree`
```go
type PrettyCfg struct {
    LineWidth int
    TabWidth int
    UseTabs bool
    Simplify bool
    Align PrettyAlignMode
    Case func(string) string  // Case transformation function
    JSONFmt bool
}

func DefaultPrettyCfg() PrettyCfg
// Returns default configuration

func (cfg PrettyCfg).Pretty(ast) (string, error)
// Applies pretty-printing to AST

type PrettyAlignMode int
// Values: 0=PrettyNoAlign, 1=PrettyPartialAlignAndDeindent, 
//         2=PrettyAlignAndDeindent, 3=PrettyOther
```

#### Subpackage: `pkg/util/json`
```go
json.ParseJSON(s string) (json.JSON, error)
// Parse JSON string to internal representation

type json.JSON
// JSON AST node with methods:
//   ObjectIter() (Iterator, error)
//   FetchValIdx(i int) (json.JSON, error)
//   Len() int
//   String() string
```

#### Subpackage: `pkg/util/pretty`
```go
type pretty.Doc
// Abstract document representation

pretty.Text(s string) Doc
pretty.Concat(...Doc) Doc
pretty.Join(sep string, ...Doc) Doc
pretty.NestUnder(label Doc, content Doc) Doc
pretty.BracketDoc(left, center, right Doc) Doc

pretty.Pretty(doc Doc, width int, useTabs bool, tabWidth int, ?) string
// Renders Doc to formatted string
```

### 3.3 Rust Equivalents Needed

| Go Package | Rust Equivalent | Notes |
|---|---|---|
| `regexp` | `regex` crate | Regex matching; requires regex compilation |
| `strings` | std::string + itertools | String operations |
| `unicode` | `unicode-general-category` or std::char | Character ops |
| `encoding/json` | `serde_json` | JSON serialization |
| `fmt` | Rust macros (println!, etc.) | Built-in formatting |
| `html/template` | `askama` or `tera` | Template engine |
| `io/ioutil` | std::fs, std::io | File/IO operations |
| `log` | `log` + `env_logger` | Logging framework |
| `math/rand` | `rand` crate | Random generation |
| `net/http` | `axum` or `actix-web` | HTTP framework |
| `os` | std::env, std::process | Environment + process |
| `os/signal` | `signal-hook` | Signal handling |
| `strconv` | std::str::FromStr | Parsing primitives |
| `sync` | std::sync::Mutex, Arc | Synchronization |
| `syscall` | libc or system-level APIs | System calls |
| `cockroachdb-parser` | **sqlparser-rs** (primary dependency) | SQL parsing |
| `kelseyhightower/envconfig` | `envconfig` crate | Config from env |
| `spf13/pflag` | `clap` (v4 with derive) | CLI argument parsing |
| `x/crypto` | `rustls` + `x509-parser` | TLS/certificate handling |

### 3.4 Critical Dependency: sqlparser-rs

**Current State**: Rust SQL parser exists in community
**Requirements for sqlfmt port**:
1. Provides PostgreSQL dialect support
2. Generates AST that pretty-printer can traverse
3. Has equivalent to CockroachDB's `tree.PrettyCfg`
4. Supports the pretty-printer algorithm

**Key Integration Points**:
- Replace `parser.Parse()` with sqlparser-rs parsing
- Create Rust equivalent of `tree.PrettyCfg` structure
- Implement pretty-printer based on `pkg/util/pretty` API
- Ensure AST structure compatible with existing logic

---

## 4. TEST CASES

### 4.1 Test Files Structure

Only **2 test files** exist in `tests/` directory:

1. **test-distributed-by.sql** (1,832 bytes)
   - Input: SQL with DISTRIBUTED BY and PARTITION BY clauses
   - Contains 103 lines of test SQL

2. **test-distributed-by.expected** (1,611 bytes)
   - Expected output: Formatted version of test input
   - Contains 55 lines of expected formatted output

### 4.2 What test-distributed-by Validates

**Features tested**:
1. ✅ DISTRIBUTED BY clause preservation
2. ✅ PARTITION BY clause preservation  
3. ✅ WITH clause formatting and compression
4. ✅ Multi-statement handling (DROP, CREATE, INSERT, UPDATE, DELETE)
5. ✅ Comment preservation and formatting
6. ✅ Case conversion (lowercase → UPPERCASE)
7. ✅ Indentation and alignment
8. ✅ TEXT type preservation (the b text field)
9. ✅ Line width aware compression

### 4.3 Test Execution Method

**Manual testing** (no automated test framework):
```bash
./sqlfmt < tests/test-distributed-by.sql | diff - tests/test-distributed-by.expected
```

**Process**:
1. Pipe input SQL to formatter
2. Diff output against expected file
3. Manual inspection if diffs exist

### 4.4 Key Test Cases Covered

| Test Case | Input | Expected Behavior |
|---|---|---|
| Basic CREATE TABLE | lowercase create | Uppercase, single-line if fits |
| WITH clause single-line | `with(appendonly=true,...)` (fits) | Compressed to: `WITH ( APPENDONLY = true, ... )` |
| WITH clause multi-line | WITH(...) with many options | Preserved as multi-line when > 80 chars |
| DISTRIBUTED BY | `distributed by(a)` | Becomes `DISTRIBUTED BY (a)` on new line |
| PARTITION BY | `partition by list(a)` | Becomes `PARTITION BY list(a)` on new line |
| Multiple statements | INSERT, UPDATE, DELETE | All processed with proper formatting |
| Comments | `-- comment` | Preserved with original intent |
| TEXT type | `b text` | Preserved as `TEXT` (not converted to STRING) |

---

## 5. ARCHITECTURE PATTERNS TO PORT

### 5.1 Overall Flow

```
Input SQL
    ↓
[stripTextType] → Count TEXT occurrences
    ↓
[extractAndStripClauses] → Extract special clauses, return clean SQL
    ↓
[process statements] → Split by semicolon, handle comments
    ↓
[parse & pretty] → Use CockroachDB parser → tree.Pretty()
    ↓
[restoreTextType] → Replace first N STRINGs with TEXT
    ↓
[restoreAllSpecialClauses] → Reattach WITH/DISTRIBUTED/PARTITION clauses
    ↓
Formatted SQL
```

### 5.2 Data Flow in Special Clause Handling

```
CREATE TABLE foo (a int) WITH (...) DISTRIBUTED BY (a);
    ↓
[extract] → Store { WITH: [...], DISTRIBUTED: [...] }
    ↓
CREATE TABLE foo (a int);  ← cleaned SQL
    ↓
[parse & format] 
    ↓
CREATE TABLE foo (a int);
    ↓
[restore] → Append clauses in order
    ↓
CREATE TABLE foo (a int)
WITH (...)
DISTRIBUTED BY (a);
```

### 5.3 Configuration Structure

```
PrettyCfg:
├─ LineWidth (int)
├─ TabWidth (int)
├─ UseTabs (bool)
├─ Simplify (bool)
├─ Align (enum: 0-3)
├─ Case (function: string→string)
└─ JSONFmt (bool)
```

### 5.4 CLI to Config Mapping Strategy

```
Flag values → Validation → Transform → PrettyCfg
```

Example transformations:
- `use-spaces: false` → `UseTabs: true` (inverted)
- `align: true` → `Align: PrettyAlignAndDeindent`
- `casemode: "upper"` → `Case: strings.ToUpper`

---

## 6. IMPLEMENTATION PRIORITY

### Phase 1: Core Library
1. Implement `FmtSQL()` core algorithm
2. Implement regex patterns and TEXT type restoration
3. Implement special clause extraction/restoration
4. Implement comment handling
5. Implement JSON formatting

### Phase 2: SQL Parser Integration
1. Evaluate sqlparser-rs capabilities
2. Adapt AST to pretty-printer algorithm
3. Implement `tree.PrettyCfg` equivalent
4. Implement pretty-printing algorithm

### Phase 3: CLI
1. Implement flag parsing (clap)
2. Implement stdin/stdout handling
3. Implement validation and error handling
4. Test with test cases

### Phase 4: HTTP Server
1. Implement HTTP server (axum)
2. Implement `/fmt` endpoint with caching
3. Implement templating for UI pages
4. Implement case transformations
5. Implement environment-based mode selection

### Phase 5: Build & Distribution
1. WASM compilation (wasm-pack)
2. Static binary compilation
3. GoReleaser equivalent (cargo-dist)

---

## 7. GOTCHAS & EDGE CASES

### 7.1 TEXT Type Restoration Limitations
- **Issue**: Simple counting approach may fail if:
  - Input has both TEXT and STRING types
  - Parser adds additional STRING types
  - Multiple TEXT→STRING conversions occur
- **Solution**: Consider tracking positions instead of just counts

### 7.2 Comment Handling Complexity
- Comments must be preserved exactly
- Newline count capped at 2
- Trailing whitespace handling tricky
- Statement parsing must account for comments before keywords

### 7.3 Special Clause Ordering
- WITH comes before DISTRIBUTED BY
- DISTRIBUTED BY comes before PARTITION BY
- Order is critical for CockroachDB/Greenplum compatibility

### 7.4 WITH Clause Compression Heuristic
- Current logic: compressed_len ≤ lineWidth?
- May need refinement for edge cases
- Original stored to fallback if compression fails

### 7.5 Parser API Differences
- Go's `parser.SplitFirstStatement()` has specific return semantics
- Rust parser may have different statement splitting behavior
- Need careful testing with multi-statement inputs

### 7.6 LRU Cache Semantics
- Go implementation clears entire cache when full
- Not true LRU; consider implementing proper LRU
- Thread safety critical for HTTP server

---

## 8. FILE STRUCTURE FOR RUST PORT

**Proposed layout**:
```
src/
├─ lib.rs                    # Public API: fmt_sql(), fmt_json()
├─ core.rs                   # Core formatting logic
├─ patterns.rs               # Regex patterns
├─ text_type.rs              # TEXT type handling
├─ special_clauses.rs        # WITH/DISTRIBUTED/PARTITION extraction
├─ config.rs                 # PrettyCfg equivalent
├─ pretty.rs                 # Pretty-printer algorithm
│
├─ cli/
│   ├─ mod.rs                # CLI module
│   └─ main.rs               # Entry point
│
├─ server/
│   ├─ mod.rs                # HTTP server module
│   ├─ handlers.rs           # Endpoint handlers
│   ├─ cache.rs              # LRU cache implementation
│   └─ templates.rs          # HTML templates
│
└─ wasm/
    └─ lib.rs                # WASM bindings
```

---

## Summary

This project is a **structured, regex-heavy SQL formatter** with the following key aspects:

1. **Core challenge**: Special clause extraction/restoration logic is intricate
2. **Parser dependency**: Heavily relies on CockroachDB AST API
3. **Limited testing**: Only 2 SQL test files; manual verification needed
4. **Dual-mode binary**: CLI + HTTP server in same codebase
5. **Pretty-printer algorithm**: Based on academic paper; must be faithfully ported
6. **Regex patterns**: 4 main patterns; fairly straightforward to port to Rust

**Estimated complexity**: Medium-High due to:
- Parser API differences
- Pretty-printer algorithm implementation
- Special clause handling logic
- HTTP server threading & caching

**Estimated effort**: 200-400 developer-hours for complete, production-ready port
