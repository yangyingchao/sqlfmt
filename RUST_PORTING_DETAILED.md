# sqlfmt Rust Port - Detailed Implementation Guide

This document provides code-level details for porting sqlfmt from Go to Rust.

---

## 1. CORE ALGORITHM: FmtSQL Walkthrough

### Go Implementation Flow

```go
func FmtSQL(cfg tree.PrettyCfg, stmts []string) (string, error) {
    var prettied strings.Builder
    var allSpecialClauses []map[string][]string
    totalTextCount := 0
    
    for _, stmt := range stmts {
        // STEP 1: Count TEXT types
        stmt, textCount := stripTextType(stmt)
        totalTextCount += textCount
        
        // STEP 2: Extract special clauses
        stmt, allSpecialClauses = extractAndStripClauses(stmt, allSpecialClauses)
        
        // STEP 3: Process statements
        for len(stmt) > 0 {
            stmt = strings.TrimSpace(stmt)
            hasContent := false
            
            // STEP 3a: Handle comments
            for {
                found := ignoreComments.FindString(stmt)
                if found == "" { break }
                prettied.WriteString(strings.TrimRightFunc(found, unicode.IsSpace))
                newlines := strings.Count(found, "\n")
                if newlines > 2 { newlines = 2 }
                prettied.WriteString(strings.Repeat("\n", newlines))
                stmt = stmt[len(found):]
                hasContent = true
            }
            
            // STEP 3b: Split first statement
            next := stmt
            if pos, _ := parser.SplitFirstStatement(stmt); pos > 0 {
                next = stmt[:pos]
                stmt = stmt[pos:]
            } else {
                stmt = ""
            }
            
            // STEP 3c: Parse and pretty-print
            allParsed, err := parser.Parse(next)
            if err != nil { return "", err }
            for _, parsed := range allParsed {
                pretty, err := cfg.Pretty(parsed.AST)
                if err != nil { return "", err }
                prettied.WriteString(pretty)
                prettied.WriteString(";\n")
                hasContent = true
            }
            if hasContent {
                prettied.WriteString("\n")
            }
        }
    }
    
    result := strings.TrimRightFunc(prettied.String(), unicode.IsSpace)
    
    // STEP 4: Restore TEXT types
    result = restoreTextType(result, totalTextCount)
    
    // STEP 5: Restore special clauses
    result = restoreAllSpecialClauses(result, allSpecialClauses, cfg.LineWidth)
    
    return result, nil
}
```

### Rust Equivalent Structure

```rust
pub fn fmt_sql(cfg: &PrettyCfg, stmts: &[String]) -> Result<String, Box<dyn Error>> {
    let mut prettied = String::new();
    let mut all_special_clauses: Vec<HashMap<String, Vec<String>>> = Vec::new();
    let mut total_text_count = 0;
    
    for stmt in stmts {
        // STEP 1: Count TEXT types
        let (stmt, text_count) = strip_text_type(stmt);
        total_text_count += text_count;
        
        // STEP 2: Extract special clauses
        let (stmt, all_special_clauses) = extract_and_strip_clauses(
            &stmt, 
            all_special_clauses
        );
        
        // STEP 3: Process statements
        let mut remaining = stmt.as_str();
        while !remaining.is_empty() {
            remaining = remaining.trim();
            let mut has_content = false;
            
            // STEP 3a: Handle comments
            loop {
                if let Some(found) = IGNORE_COMMENTS.find(remaining) {
                    let comment = found.as_str();
                    prettied.push_str(comment.trim_end());
                    
                    let newlines = comment.matches('\n').count().min(2);
                    prettied.push_str(&"\n".repeat(newlines));
                    
                    remaining = &remaining[found.end()..];
                    has_content = true;
                } else {
                    break;
                }
            }
            
            // STEP 3b: Split first statement
            let (next, next_remaining) = split_first_statement(remaining)?;
            remaining = next_remaining;
            
            // STEP 3c: Parse and pretty-print
            let parsed = parse_sql(&next)?;
            for ast in parsed {
                let pretty = cfg.pretty(&ast)?;
                prettied.push_str(&pretty);
                prettied.push_str(";\n");
                has_content = true;
            }
            
            if has_content {
                prettied.push('\n');
            }
        }
    }
    
    let mut result = prettied.trim_end().to_string();
    
    // STEP 4: Restore TEXT types
    result = restore_text_type(&result, total_text_count);
    
    // STEP 5: Restore special clauses
    result = restore_all_special_clauses(&result, &all_special_clauses, cfg.line_width);
    
    Ok(result)
}
```

---

## 2. SPECIAL CLAUSE EXTRACTION DETAILED

### Data Flow Example

**Input SQL**:
```sql
CREATE TABLE r (a int4, b text)
WITH (appendonly = true, compresslevel = 3)
DISTRIBUTED BY (a);
```

### Step-by-Step Processing

```
1. Match CREATE TABLE pattern:
   "CREATE TABLE r (a int4, b text) WITH (appendonly = true, compresslevel = 3) DISTRIBUTED BY (a);"

2. Extract WITH:
   Original: "WITH (appendonly = true, compresslevel = 3)"
   Compressed: "WITH (appendonly = true, compresslevel = 3)" [same, fits in 80 chars]
   
3. Extract DISTRIBUTED:
   Value: "DISTRIBUTED BY (a)"
   
4. Extract PARTITION:
   Value: "" [not present]
   
5. Build clause map:
   {
     "WITH": ["WITH (appendonly = true, compresslevel = 3)"],
     "WITH_ORIGINAL": ["WITH (appendonly = true, compresslevel = 3)"],
     "DISTRIBUTED": ["DISTRIBUTED BY (a)"],
     "PARTITION": [""]
   }
   
6. Strip all clauses from SQL:
   "CREATE TABLE r (a int4, b text);"
```

### Go Implementation

```go
func extractAndStripClauses(sql string, allClauses []map[string][]string) (string, []map[string][]string) {
    result := sql
    createTablePattern := regexp.MustCompile(`(?i)CREATE\s+TABLE\s+[^;]+?;`)
    
    matches := createTablePattern.FindAllString(result, -1)
    for _, match := range matches {
        withMatches := withClausePattern.FindAllString(match, -1)
        withClauseOriginal := ""
        withClauseCompressed := ""
        if len(withMatches) > 0 {
            withClauseOriginal = withMatches[0]
            withClauseOriginal = regexp.MustCompile(`^\s*`).ReplaceAllString(withClauseOriginal, "")
            
            withClauseCompressed = withMatches[0]
            withClauseCompressed = regexp.MustCompile(`\s+`).ReplaceAllString(withClauseCompressed, " ")
            withClauseCompressed = strings.TrimSpace(withClauseCompressed)
        }
        
        distMatches := distributedByPattern.FindAllString(match, -1)
        distClause := ""
        if len(distMatches) > 0 {
            distClause = strings.TrimSpace(distMatches[0])
        }
        
        partitionMatches := partitionByPattern.FindAllString(match, -1)
        partitionClause := ""
        if len(partitionMatches) > 0 {
            partitionClause = strings.TrimSpace(partitionMatches[0])
        }
        
        clauses := map[string][]string{
            "WITH":          {withClauseCompressed},
            "WITH_ORIGINAL": {withClauseOriginal},
            "DISTRIBUTED":   {distClause},
            "PARTITION":     {partitionClause},
        }
        allClauses = append(allClauses, clauses)
    }
    
    result = withClausePattern.ReplaceAllString(result, "")
    result = distributedByPattern.ReplaceAllString(result, "")
    result = partitionByPattern.ReplaceAllString(result, "")
    
    return result, allClauses
}
```

### Rust Equivalent

```rust
fn extract_and_strip_clauses(
    sql: &str, 
    mut all_clauses: Vec<HashMap<String, Vec<String>>>
) -> (String, Vec<HashMap<String, Vec<String>>>) {
    let mut result = sql.to_string();
    let create_table_regex = regex::Regex::new(r"(?i)CREATE\s+TABLE\s+[^;]+?;").unwrap();
    
    for capture in create_table_regex.find_iter(sql) {
        let table_def = capture.as_str();
        
        // Extract WITH clause
        let mut with_clause_original = String::new();
        let mut with_clause_compressed = String::new();
        if let Some(with_match) = WITH_CLAUSE_PATTERN.find(table_def) {
            with_clause_original = with_match.as_str().to_string();
            with_clause_original = with_clause_original.trim_start().to_string();
            
            with_clause_compressed = with_match.as_str().to_string();
            // Compress whitespace
            while with_clause_compressed.contains("  ") {
                with_clause_compressed = with_clause_compressed.replace("  ", " ");
            }
            with_clause_compressed = with_clause_compressed.trim().to_string();
        }
        
        // Extract DISTRIBUTED BY clause
        let mut dist_clause = String::new();
        if let Some(dist_match) = DISTRIBUTED_BY_PATTERN.find(table_def) {
            dist_clause = dist_match.as_str().trim().to_string();
        }
        
        // Extract PARTITION BY clause
        let mut partition_clause = String::new();
        if let Some(part_match) = PARTITION_BY_PATTERN.find(table_def) {
            partition_clause = part_match.as_str().trim().to_string();
        }
        
        // Build clause map
        let mut clauses = HashMap::new();
        clauses.insert("WITH".to_string(), vec![with_clause_compressed]);
        clauses.insert("WITH_ORIGINAL".to_string(), vec![with_clause_original]);
        clauses.insert("DISTRIBUTED".to_string(), vec![dist_clause]);
        clauses.insert("PARTITION".to_string(), vec![partition_clause]);
        
        all_clauses.push(clauses);
    }
    
    // Strip clauses from result
    result = WITH_CLAUSE_PATTERN.replace_all(&result, "").to_string();
    result = DISTRIBUTED_BY_PATTERN.replace_all(&result, "").to_string();
    result = PARTITION_BY_PATTERN.replace_all(&result, "").to_string();
    
    (result, all_clauses)
}
```

---

## 3. SPECIAL CLAUSE RESTORATION WITH COMPRESSION

### Go Implementation

```go
func restoreAllSpecialClauses(formatted string, allClauses []map[string][]string, lineWidth int) string {
    if len(allClauses) == 0 {
        return formatted
    }

    result := formatted
    clauseIdx := 0

    createTablePattern := regexp.MustCompile(`(?i)(CREATE\s+TABLE\s+[^;]+?);`)
    result = createTablePattern.ReplaceAllStringFunc(result, func(match string) string {
        if clauseIdx >= len(allClauses) {
            return match
        }
        clauses := allClauses[clauseIdx]
        clauseIdx++

        statement := strings.TrimSuffix(match, ";")

        distClause := ""
        if len(clauses["DISTRIBUTED"]) > 0 && clauses["DISTRIBUTED"][0] != "" {
            distClause = clauses["DISTRIBUTED"][0]
        }

        // Get WITH clauses
        withClauseCompressed := ""
        withClauseOriginal := ""
        if len(clauses["WITH"]) > 0 {
            withClauseCompressed = clauses["WITH"][0]
        }
        if len(clauses["WITH_ORIGINAL"]) > 0 {
            withClauseOriginal = clauses["WITH_ORIGINAL"][0]
        }

        // Decide which WITH version to use
        withClause := withClauseCompressed
        if withClauseCompressed != "" && len(withClauseCompressed) > lineWidth {
            withClause = withClauseOriginal
        }

        // Get PARTITION clause
        partitionClause := ""
        if len(clauses["PARTITION"]) > 0 && clauses["PARTITION"][0] != "" {
            partitionClause = clauses["PARTITION"][0]
        }

        // Append clauses in order
        if withClause != "" {
            statement += "\n" + normalizeWithClause(strings.TrimLeft(withClause, " \t"))
        }
        if distClause != "" {
            statement += "\n" + normalizeDistributedByKeyword(distClause)
        }
        if partitionClause != "" {
            statement += "\n" + normalizePartitionByKeyword(partitionClause)
        }

        statement += ";"
        return statement
    })

    return result
}
```

### Rust Equivalent

```rust
fn restore_all_special_clauses(
    formatted: &str, 
    all_clauses: &[HashMap<String, Vec<String>>], 
    line_width: usize
) -> String {
    if all_clauses.is_empty() {
        return formatted.to_string();
    }

    let mut result = formatted.to_string();
    let create_table_pattern = regex::Regex::new(r"(?i)(CREATE\s+TABLE\s+[^;]+?);").unwrap();
    
    let mut clause_idx = 0;

    result = create_table_pattern.replace_all(&result, |caps: &regex::Captures| {
        if clause_idx >= all_clauses.len() {
            return caps[0].to_string();
        }
        
        let clauses = &all_clauses[clause_idx];
        clause_idx += 1;

        let mut statement = caps[1].to_string();

        // Get DISTRIBUTED clause
        let dist_clause = clauses
            .get("DISTRIBUTED")
            .and_then(|v| v.first())
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .unwrap_or("");

        // Get WITH clauses
        let with_compressed = clauses
            .get("WITH")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("");
        let with_original = clauses
            .get("WITH_ORIGINAL")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("");

        // Decide which WITH version to use
        let with_clause = if !with_compressed.is_empty() && with_compressed.len() > line_width {
            with_original
        } else {
            with_compressed
        };

        // Get PARTITION clause
        let partition_clause = clauses
            .get("PARTITION")
            .and_then(|v| v.first())
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .unwrap_or("");

        // Append clauses in order
        if !with_clause.is_empty() {
            statement.push('\n');
            statement.push_str(&normalize_with_clause(&with_clause.trim_start()));
        }
        if !dist_clause.is_empty() {
            statement.push('\n');
            statement.push_str(&normalize_distributed_by_keyword(dist_clause));
        }
        if !partition_clause.is_empty() {
            statement.push('\n');
            statement.push_str(&normalize_partition_by_keyword(partition_clause));
        }

        statement.push(';');
        statement
    }).to_string();

    result
}
```

---

## 4. TEXT TYPE HANDLING

### Problem & Solution

```
Input:  CREATE TABLE t (a TEXT, b TEXT);
Parser: CREATE TABLE t (a STRING, b STRING);
Output: CREATE TABLE t (a STRING, b STRING);  ✗ WRONG

Goal:   CREATE TABLE t (a TEXT, b TEXT);     ✓ CORRECT
```

### Go Implementation

```go
func stripTextType(sql string) (string, int) {
    matches := textTypePattern.FindAllStringIndex(sql, -1)
    textCount := len(matches)
    return sql, textCount  // Non-destructive: just count
}

func restoreTextType(formatted string, textCount int) string {
    if textCount == 0 {
        return formatted
    }

    stringPattern := regexp.MustCompile(`(?i)\bSTRING\b`)
    replaced := 0
    result := stringPattern.ReplaceAllStringFunc(formatted, func(match string) string {
        if replaced < textCount {
            replaced++
            return "TEXT"
        }
        return match
    })

    return result
}
```

### Rust Equivalent

```rust
fn strip_text_type(sql: &str) -> (String, usize) {
    let text_count = TEXT_TYPE_PATTERN.find_iter(sql).count();
    (sql.to_string(), text_count)
}

fn restore_text_type(formatted: &str, text_count: usize) -> String {
    if text_count == 0 {
        return formatted.to_string();
    }

    let mut replaced = 0;
    let string_pattern = regex::Regex::new(r"(?i)\bSTRING\b").unwrap();
    
    string_pattern.replace_all(formatted, |_: &regex::Captures| {
        if replaced < text_count {
            replaced += 1;
            "TEXT".to_string()
        } else {
            "STRING".to_string()
        }
    }).to_string()
}
```

**Issue**: This uses a closure that captures `replaced` mutably, which won't compile in Rust.

**Better Solution**:
```rust
fn restore_text_type(formatted: &str, text_count: usize) -> String {
    if text_count == 0 {
        return formatted.to_string();
    }

    let string_pattern = regex::Regex::new(r"(?i)\bSTRING\b").unwrap();
    let mut replaced = 0;
    let mut result = String::new();
    let mut last_end = 0;

    for mat in string_pattern.find_iter(formatted) {
        result.push_str(&formatted[last_end..mat.start()]);
        
        if replaced < text_count {
            result.push_str("TEXT");
            replaced += 1;
        } else {
            result.push_str("STRING");
        }
        
        last_end = mat.end();
    }
    
    result.push_str(&formatted[last_end..]);
    result
}
```

---

## 5. REGEX PATTERNS

### Go Patterns

```go
var (
    ignoreComments        = regexp.MustCompile(`^--.*\s*`)
    distributedByPattern  = regexp.MustCompile(`(?i)\s+distributed\s+by\s*\([^)]*\)`)
    partitionByPattern    = regexp.MustCompile(`(?i)\s+partition\s+by\s+\w+\s*\([^)]*\)`)
    withClausePattern     = regexp.MustCompile(`(?i)\s+with\s*\([^)]*\)`)
    textTypePattern       = regexp.MustCompile(`(?i)\bTEXT\b`)
)
```

### Rust Equivalents

```rust
lazy_static! {
    static ref IGNORE_COMMENTS: regex::Regex = 
        regex::Regex::new(r"^--.*\s*").unwrap();
    
    static ref DISTRIBUTED_BY_PATTERN: regex::Regex = 
        regex::Regex::new(r"(?i)\s+distributed\s+by\s*\([^)]*\)").unwrap();
    
    static ref PARTITION_BY_PATTERN: regex::Regex = 
        regex::Regex::new(r"(?i)\s+partition\s+by\s+\w+\s*\([^)]*\)").unwrap();
    
    static ref WITH_CLAUSE_PATTERN: regex::Regex = 
        regex::Regex::new(r"(?i)\s+with\s*\([^)]*\)").unwrap();
    
    static ref TEXT_TYPE_PATTERN: regex::Regex = 
        regex::Regex::new(r"(?i)\bTEXT\b").unwrap();
}
```

Or using `once_cell`:
```rust
use once_cell::sync::Lazy;

static IGNORE_COMMENTS: Lazy<regex::Regex> = 
    Lazy::new(|| regex::Regex::new(r"^--.*\s*").unwrap());
// ... etc
```

---

## 6. CONFIGURATION STRUCTURE

### Go Version

```go
type PrettyCfg struct {
    LineWidth int
    TabWidth int
    UseTabs bool
    Simplify bool
    Align PrettyAlignMode  // enum-like
    Case func(string) string
    JSONFmt bool
}
```

### Rust Version

```rust
#[derive(Clone)]
pub struct PrettyCfg {
    pub line_width: usize,
    pub tab_width: usize,
    pub use_tabs: bool,
    pub simplify: bool,
    pub align: PrettyAlignMode,
    pub case_fn: fn(&str) -> String,
    pub json_fmt: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PrettyAlignMode {
    NoAlign = 0,
    PartialAlignAndDeindent = 1,
    AlignAndDeindent = 2,
    Other = 3,
}

impl Default for PrettyCfg {
    fn default() -> Self {
        PrettyCfg {
            line_width: 80,
            tab_width: 4,
            use_tabs: true,
            simplify: true,
            align: PrettyAlignMode::NoAlign,
            case_fn: |s| s.to_uppercase(),
            json_fmt: true,
        }
    }
}
```

---

## 7. CLI ARGUMENT MAPPING

### Go to Rust Mapping Table

| Go Flag | Type | Go Code | Rust Equivalent |
|---|---|---|---|
| `--print-width` | int | `cfg.LineWidth = *flagPrintWidth` | `cfg.line_width = args.print_width` |
| `--tab-width` | int | `cfg.TabWidth = *flagTabWidth` | `cfg.tab_width = args.tab_width` |
| `--use-spaces` | bool | `cfg.UseTabs = !*flagUseSpaces` | `cfg.use_tabs = !args.use_spaces` |
| `--no-simplify` | bool | `cfg.Simplify = !*flagNoSimplify` | `cfg.simplify = !args.no_simplify` |
| `--align` | bool | `cfg.Align = tree.PrettyAlignAndDeindent` | `cfg.align = PrettyAlignMode::AlignAndDeindent` |
| `--casemode` | str | `cfg.Case = caseModes[*flagCasemode]` | `cfg.case_fn = get_case_fn(args.case_mode)` |

### Case Function Mapping

```rust
fn get_case_fn(mode: &str) -> fn(&str) -> String {
    match mode {
        "upper" => |s| s.to_uppercase(),
        "lower" => |s| s.to_lowercase(),
        "title" => |s| {
            s.to_lowercase()
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        },
        "spongebob" => |s| {
            s.chars()
                .map(|c| {
                    if rand::random() {
                        c.to_uppercase().to_string()
                    } else {
                        c.to_lowercase().to_string()
                    }
                })
                .collect()
        },
        _ => |s| s.to_uppercase(),
    }
}
```

---

## 8. KEYWORD NORMALIZATION

### Go Implementation

```go
func normalizeWithClause(clause string) string {
    pattern := regexp.MustCompile(`(?i)^\s*with`)
    result := pattern.ReplaceAllStringFunc(clause, func(match string) string {
        leadingSpace := regexp.MustCompile(`^\s*`).FindString(match)
        return leadingSpace + "WITH"
    })
    
    paramPattern := regexp.MustCompile(`\b([a-zA-Z_]\w*)\s*=`)
    result = paramPattern.ReplaceAllStringFunc(result, func(match string) string {
        paramMatch := regexp.MustCompile(`([a-zA-Z_]\w*)\s*=`).FindStringSubmatch(match)
        if len(paramMatch) > 1 {
            return strings.ToUpper(paramMatch[1]) + " ="
        }
        return match
    })
    
    return result
}
```

### Rust Implementation

```rust
fn normalize_with_clause(clause: &str) -> String {
    // Match leading whitespace and "with"
    let with_pattern = regex::Regex::new(r"(?i)^\s*with").unwrap();
    let result = with_pattern.replace(clause, |caps: &regex::Captures| {
        let m = caps[0].as_bytes();
        let leading_space_len = m.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
        let leading_space = std::str::from_utf8(&m[..leading_space_len]).unwrap_or("");
        format!("{}WITH", leading_space)
    });
    
    // Normalize parameter names
    let param_pattern = regex::Regex::new(r"\b([a-zA-Z_]\w*)\s*=").unwrap();
    param_pattern.replace_all(&result, |caps: &regex::Captures| {
        format!("{} =", caps[1].to_uppercase())
    }).to_string()
}

fn normalize_distributed_by_keyword(clause: &str) -> String {
    let pattern = regex::Regex::new(r"(?i)(distributed)\s+(by)").unwrap();
    pattern.replace_all(clause, "DISTRIBUTED BY").to_string()
}

fn normalize_partition_by_keyword(clause: &str) -> String {
    let pattern = regex::Regex::new(r"(?i)(partition)\s+(by)").unwrap();
    pattern.replace_all(clause, "PARTITION BY").to_string()
}
```

---

## 9. COMMENT PRESERVATION

### Go Implementation

```go
for {
    found := ignoreComments.FindString(stmt)
    if found == "" {
        break
    }
    // Remove trailing whitespace but keep up to 2 newlines
    prettied.WriteString(strings.TrimRightFunc(found, unicode.IsSpace))
    newlines := strings.Count(found, "\n")
    if newlines > 2 {
        newlines = 2
    }
    prettied.WriteString(strings.Repeat("\n", newlines))
    stmt = stmt[len(found):]
    hasContent = true
}
```

### Rust Implementation

```rust
loop {
    if let Some(mat) = IGNORE_COMMENTS.find(remaining) {
        let comment = mat.as_str();
        
        // Trim trailing whitespace
        let trimmed = comment.trim_end();
        prettied.push_str(trimmed);
        
        // Cap newlines at 2
        let newline_count = comment.matches('\n').count().min(2);
        for _ in 0..newline_count {
            prettied.push('\n');
        }
        
        remaining = &remaining[mat.end()..];
        has_content = true;
    } else {
        break;
    }
}
```

---

## 10. HTTP SERVER CACHING

### Go Implementation

```go
var cache = struct {
    sync.RWMutex
    m map[string]fmtResponse
}{
    m: make(map[string]fmtResponse),
}

func Fmt(w http.ResponseWriter, r *http.Request) fmtResponse {
    cache.RLock()
    hit, ok := cache.m[r.URL.RawQuery]
    cache.RUnlock()
    if ok {
        return hit
    }

    res, err := fmtSQLRequest(r)
    response := fmtResponse{
        Data:  res,
        Error: err != nil,
    }
    if err != nil {
        response.Data = err.Error()
    }
    
    cache.Lock()
    if len(cache.m) > 10000 {
        // Clear entire cache when full
        for k := range cache.m {
            delete(cache.m, k)
        }
    }
    cache.m[r.URL.RawQuery] = response
    cache.Unlock()
    
    return response
}
```

### Rust Implementation (with moka crate)

```rust
use moka::future::Cache;
use std::sync::Arc;

#[derive(Clone)]
pub struct CachedFormatter {
    cache: Arc<Cache<String, FmtResponse>>,
}

impl CachedFormatter {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .build();
        
        CachedFormatter {
            cache: Arc::new(cache),
        }
    }

    pub async fn format(&self, query: &str, config: &PrettyCfg) -> FmtResponse {
        // Check cache
        if let Some(cached) = self.cache.get(query).await {
            return cached;
        }

        // Format
        let response = match fmt_sql_request(query, config).await {
            Ok(data) => FmtResponse {
                data,
                error: false,
            },
            Err(e) => FmtResponse {
                data: e.to_string(),
                error: true,
            },
        };

        // Cache result
        self.cache.insert(query.to_string(), response.clone()).await;

        response
    }
}
```

---

## 11. STATEMENT SPLITTING

### Go Implementation

```go
// SplitFirstStatement returns position of end of first statement
if pos, _ := parser.SplitFirstStatement(stmt); pos > 0 {
    next = stmt[:pos]
    stmt = stmt[pos:]
} else {
    stmt = ""
}
```

### Rust Implementation

For sqlparser-rs or similar, statements are typically parsed directly:

```rust
fn split_first_statement(sql: &str) -> Result<(String, &str)> {
    // Try to find semicolon
    if let Some(pos) = sql.find(';') {
        Ok((sql[..=pos].to_string(), sql[pos+1..].trim_start()))
    } else {
        Ok((sql.to_string(), ""))
    }
}
```

Or with a proper parser:
```rust
fn split_first_statement(sql: &str) -> Result<(String, &str)> {
    match sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, sql) {
        Ok(statements) => {
            if statements.is_empty() {
                Ok((String::new(), ""))
            } else {
                // Find position of first statement by parsing
                let first_stmt = statements[0].to_string();
                // Find semicolon position
                if let Some(pos) = sql.find(&first_stmt) {
                    let end = pos + first_stmt.len();
                    Ok((sql[..end].to_string(), sql[end..].trim_start()))
                } else {
                    Ok((first_stmt, ""))
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}
```

---

## Summary of Key Patterns

| Pattern | Go | Rust |
|---|---|---|
| **Regex matching** | `regexp.MustCompile()` | `once_cell::Lazy<regex::Regex>` |
| **String building** | `strings.Builder` | `String::push_str()` |
| **String replacement** | `regexp.ReplaceAllString()` | `regex.replace_all()` |
| **Case conversion** | `strings.ToUpper()` | `str.to_uppercase()` |
| **Trimming** | `strings.Trim*Func()` | `str.trim*()` |
| **Configuration** | Struct with methods | Struct with associated functions |
| **Caching** | `sync.Map` + `sync.RWMutex` | `moka::Cache` or `Arc<Mutex<HashMap>>` |
| **Error handling** | `error` interface | `Result<T, E>` |

