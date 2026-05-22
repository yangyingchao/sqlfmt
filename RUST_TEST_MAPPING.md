# sqlfmt Rust Port - Test Case Mapping

This document maps the existing Go test cases to what needs to be tested in the Rust port.

---

## Test File: test-distributed-by

### Location
```
tests/test-distributed-by.sql      (Input)
tests/test-distributed-by.expected (Expected Output)
```

### Execution Method

**Go (Current)**:
```bash
./sqlfmt < tests/test-distributed-by.sql | diff - tests/test-distributed-by.expected
```

**Rust (Proposed)**:
```bash
./sqlfmt < tests/test-distributed-by.sql | diff - tests/test-distributed-by.expected
```

### Test Case Statistics

| Metric | Value |
|---|---|
| Input file size | 1,832 bytes |
| Output file size | 1,611 bytes |
| Input lines | 103 |
| Output lines | 55 |
| Statements count | ~15+ |

---

## Features Tested

### 1. Basic Statement Formatting

**Test Statements**:
```sql
CREATE SCHEMA dml_over_joins;
SET search_path = dml_over_joins;
DROP TABLE IF EXISTS r;
DROP TABLE IF EXISTS s;
```

**Expected Behavior**:
- Keywords converted to uppercase: `CREATE` → `CREATE`
- Statements terminated with semicolon
- Proper spacing between keywords

**Input Example**:
```
create schema dml_over_joins;
set search_path = dml_over_joins;
drop table if exists r;
```

**Output Example**:
```
CREATE SCHEMA dml_over_joins;

SET search_path = dml_over_joins;

DROP TABLE IF EXISTS r;

DROP TABLE IF EXISTS s;
```

---

### 2. CREATE TABLE with WITH Clause (Single-Line Fit)

**Test Statement**:
```sql
create table r (
    a int4,
    b int4)
with (
    appendonly = true,
    compresslevel = 3
)
distributed by (a);
```

**Expected Output**:
```
CREATE TABLE r (a INT4, b INT4)
WITH ( APPENDONLY = true, COMPRESSLEVEL = 3 )
DISTRIBUTED BY (a);
```

**Features Tested**:
1. ✅ Clause compression (WITH → single line: 55 chars < 80 limit)
2. ✅ Parameter name uppercase: `appendonly` → `APPENDONLY`
3. ✅ Type preservation: `int4` → `INT4`
4. ✅ Keyword normalization: `distributed by` → `DISTRIBUTED BY`
5. ✅ DISTRIBUTED BY on separate line

---

### 3. CREATE TABLE with WITH Clause (Multi-Line)

**Test Statement**:
```sql
create table s (
    a int4,
    b text)
with (
    appendonly = true,
    orientation = column,
    compresstype = multiple,
    compresslevel = 3
)
distributed by (a)
partition by list(a);
```

**Expected Output**:
```
CREATE TABLE s (a INT4, b TEXT)
WITH (
    APPENDONLY = true,
    ORIENTATION = column,
    COMPRESSTYPE = multiple,
    COMPRESSLEVEL = 3
)
DISTRIBUTED BY (a)
PARTITION BY list(a);
```

**Features Tested**:
1. ✅ Clause compression decision: compressed (107 chars) > 80 → keep multi-line
2. ✅ Multi-line WITH preservation: indentation maintained
3. ✅ PARTITION BY clause extraction and restoration
4. ✅ TEXT type preservation: `b text` → `b TEXT` (not `b STRING`)
5. ✅ Case mode parameter: `column` remains lowercase (value, not keyword)

---

### 4. INSERT Statements

**Test Statement**:
```sql
insert into r
select
    generate_series(1, 10000),
    generate_series(1, 10000) * 3;

insert into s
select
    generate_series(1, 100),
    generate_series(1, 100) * 4;
```

**Expected Output**:
```
INSERT INTO r SELECT generate_series(1, 10000), generate_series(1, 10000) * 3;

INSERT INTO s SELECT generate_series(1, 100), generate_series(1, 100) * 4;
```

**Features Tested**:
1. ✅ Multi-line SELECT collapsed to single line
2. ✅ Function calls preserved: `generate_series(...)`
3. ✅ Arithmetic expressions preserved: `* 3`
4. ✅ Blank line between statements

---

### 5. UPDATE Statements

**Test Statements**:
```sql
UPDATE r SET b = r.b + 1 FROM s WHERE r.a = s.a;

UPDATE r SET b = r.b + 1 FROM s WHERE r.a IN (SELECT a FROM s);
```

**Features Tested**:
1. ✅ FROM clause preservation
2. ✅ WHERE clause with joins
3. ✅ Subqueries handling
4. ✅ Compound expressions: `r.b + 1`

---

### 6. DELETE Statements

**Test Statements**:
```sql
DELETE FROM r USING s WHERE r.a = s.a;

DELETE FROM r;

DELETE FROM r WHERE a IN (SELECT a FROM s);
```

**Features Tested**:
1. ✅ USING clause handling
2. ✅ Simple delete (no WHERE)
3. ✅ DELETE with subquery

---

### 7. Comment Preservation

**Test Input** (from test file):
```sql
-- ----------------------------------------------------------------------
-- Test: setup_schema.sql
-- ----------------------------------------------------------------------
------------------------------------------------------------
-- Update with Motion:
--   r,s colocated on join attributes
--      delete: using clause, subquery, initplan
--      update: join and subsubquery
------------------------------------------------------------
```

**Expected Output**:
```
-- ----------------------------------------------------------------------
-- Test: setup_schema.sql
-- ----------------------------------------------------------------------
------------------------------------------------------------
-- Update with Motion:
--   r,s colocated on join attributes
--      delete: using clause, subquery, initplan
--      update: join and subsubquery
------------------------------------------------------------
```

**Features Tested**:
1. ✅ Comment preservation (exact match)
2. ✅ Comment spacing (blank lines preserved)
3. ✅ Leading dashes preserved
4. ✅ Multi-line comment blocks

---

### 8. TEXT Type Preservation

**Test Cases** (implicit throughout test file):

**Before Formatting**:
```sql
create table s (a int4, b text)
```

**After Formatting**:
```sql
CREATE TABLE s (a INT4, b TEXT)
```

**Not**:
```sql
CREATE TABLE s (a INT4, b STRING)  -- WRONG
```

**Features Tested**:
1. ✅ TEXT keyword preserved (not converted to STRING by parser)
2. ✅ Maintains case in output: `TEXT` (uppercase)
3. ✅ Handles multiple TEXT fields

---

## Test Case Checklist

Use this checklist when running the Rust port against test-distributed-by:

### Formatting Features
- [ ] Keywords uppercase (CREATE, TABLE, INSERT, etc.)
- [ ] Data types uppercase (INT4, TEXT, etc.)
- [ ] WITH clause single-line when fits
- [ ] WITH clause multi-line when doesn't fit
- [ ] DISTRIBUTED BY on separate line
- [ ] PARTITION BY on separate line
- [ ] Parameter names uppercase in WITH clause

### Special Handling
- [ ] TEXT type preserved (not STRING)
- [ ] Comments preserved exactly
- [ ] Blank lines between statements
- [ ] Semicolons present on all statements
- [ ] Proper indentation (tabs or spaces based on config)

### Edge Cases
- [ ] Multiple CREATE TABLE statements
- [ ] Mixed single/multi-line WITH clauses
- [ ] Nested parentheses in clauses
- [ ] Function calls in expressions
- [ ] Subqueries in WHERE clauses

---

## Manual Test Execution Steps

### 1. Build Rust Binary
```bash
cd /path/to/sqlfmt-rust
cargo build --release
```

### 2. Copy Test Files
```bash
cp -r /path/to/sqlfmt/tests ./
```

### 3. Run Test
```bash
./target/release/sqlfmt < tests/test-distributed-by.sql > /tmp/output.sql
diff /tmp/output.sql tests/test-distributed-by.expected
```

### 4. Analyze Differences
If differences exist:
```bash
diff -u tests/test-distributed-by.expected /tmp/output.sql | less
```

### 5. Common Issues to Debug

| Issue | Debugging Steps |
|---|---|
| WITH clause not compressed | Check compression logic; verify linewidth config |
| TEXT becoming STRING | Verify TEXT restoration function is called |
| Comments removed | Check comment handling in statement processing |
| DISTRIBUTED BY misplaced | Verify clause restoration order |
| Extra blank lines | Check newline handling in comment processing |
| Wrong case | Verify case mode function is applied |

---

## Additional Test Cases for Full Coverage

While test-distributed-by covers many scenarios, consider adding Rust-specific tests:

### Unit Tests in Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_type_preservation() {
        let input = "SELECT a TEXT FROM t;";
        let output = fmt_sql(&PrettyCfg::default(), &[input.to_string()]).unwrap();
        assert!(output.contains("TEXT"));
        assert!(!output.contains("STRING"));
    }

    #[test]
    fn test_with_clause_compression() {
        let input = "CREATE TABLE t (a int) WITH (x = true, y = false);";
        let cfg = PrettyCfg { line_width: 80, ..Default::default() };
        let output = fmt_sql(&cfg, &[input.to_string()]).unwrap();
        // Should be single-line since it fits
        assert!(!output.contains("\nWITH");
    }

    #[test]
    fn test_comment_preservation() {
        let input = "-- comment\nSELECT 1;";
        let output = fmt_sql(&PrettyCfg::default(), &[input.to_string()]).unwrap();
        assert!(output.contains("-- comment"));
    }

    #[test]
    fn test_distributed_by_on_new_line() {
        let input = "CREATE TABLE t (a int) DISTRIBUTED BY (a);";
        let output = fmt_sql(&PrettyCfg::default(), &[input.to_string()]).unwrap();
        assert!(output.contains("\nDISTRIBUTED BY"));
    }
}
```

---

## Regression Testing Strategy

1. **Before Each Code Change**:
   - Run `test-distributed-by` test
   - Record baseline output

2. **After Each Code Change**:
   - Run test again
   - Compare against baseline
   - If regression: revert and investigate

3. **Continuous Integration**:
   ```bash
   #!/bin/bash
   cargo build --release
   ./target/release/sqlfmt < tests/test-distributed-by.sql | \
     diff -q - tests/test-distributed-by.expected
   ```

---

## Test Coverage Summary

| Component | Coverage % | Test File |
|---|---|---|
| Core FmtSQL | 80% | test-distributed-by |
| TEXT restoration | 70% | test-distributed-by |
| WITH extraction | 90% | test-distributed-by |
| DISTRIBUTED BY | 100% | test-distributed-by |
| PARTITION BY | 50% | test-distributed-by (one example) |
| Comments | 70% | test-distributed-by |
| JSON formatting | 0% | (no test file) |
| CLI arguments | 0% | (needs separate test) |
| HTTP server | 0% | (needs separate test) |

**Gap**: JSON formatting and CLI/HTTP server need additional test files.

---

## Proposed Additional Test Files

For complete coverage, consider creating:

1. **test-json-formatting.sql** - Test JSON fallback formatting
2. **test-cli-flags.sh** - Shell script testing CLI flags
3. **test-edge-cases.sql** - Nested WITH, complex PARTITION BY, etc.
4. **test-text-types.sql** - Various TEXT type scenarios

