# sqlfmt Go to Rust Port - Complete Analysis Index

**Generated**: May 22, 2026
**Project**: sqlfmt - SQL formatter based on Wadler's pretty printer algorithm
**Goal**: Complete documentation for porting from Go to Rust

---

## 📚 Documentation Files

This analysis consists of three complementary documents:

### 1. **RUST_PORTING_ANALYSIS.md** (22 KB, 710 lines)
**High-level overview and architecture analysis**

**Contents**:
- ✅ Core library analysis (sqlfmt.go)
- ✅ CLI and HTTP server analysis (backend/main.go)
- ✅ Dependencies analysis and Rust equivalents
- ✅ Test case overview
- ✅ Architecture patterns
- ✅ Implementation priority phases
- ✅ Gotchas and edge cases
- ✅ File structure for Rust port

**Best for**: Understanding the big picture, planning the port, identifying dependencies

**Key Sections**:
- **1.1**: Exported functions (FmtSQL, FmtJSON)
- **1.2**: Regex patterns (5 main patterns documented)
- **1.3**: TEXT type restoration logic (2-phase approach)
- **1.4**: Special clause extraction/restoration flow
- **1.5**: WITH clause compression algorithm
- **2.1**: CLI flags with types and defaults (9 flags)
- **2.5**: HTTP server endpoints and caching
- **3.2**: CockroachDB parser API usage
- **3.3**: Rust equivalents for Go packages
- **3.4**: Critical dependency: sqlparser-rs

---

### 2. **RUST_PORTING_DETAILED.md** (28 KB, 978 lines)
**Code-level implementation details with Go/Rust side-by-side**

**Contents**:
- ✅ FmtSQL algorithm walkthrough (Go vs Rust)
- ✅ Special clause extraction detailed flow
- ✅ Special clause restoration with compression
- ✅ TEXT type handling (with solutions to common issues)
- ✅ Regex patterns (with Rust implementations)
- ✅ Configuration structure
- ✅ CLI argument mapping table
- ✅ Keyword normalization
- ✅ Comment preservation
- ✅ HTTP server caching
- ✅ Statement splitting
- ✅ Summary of key patterns

**Best for**: Actual implementation, copy-paste code examples, pattern matching

**Code Examples**:
- 11 major functions with Go/Rust implementations
- Configuration structure definitions
- Pattern matching strategies
- Closure and iterator handling in Rust
- Error handling patterns

---

### 3. **RUST_TEST_MAPPING.md** (11 KB, 432 lines)
**Test case analysis and coverage**

**Contents**:
- ✅ Test file locations and structure
- ✅ Detailed feature testing breakdown
- ✅ Test execution methods
- ✅ Test case checklist
- ✅ Manual test execution steps
- ✅ Debugging guide
- ✅ Unit test examples
- ✅ Regression testing strategy
- ✅ Test coverage gaps

**Best for**: Testing, quality assurance, validation

**Test Features**:
- Basic statement formatting
- CREATE TABLE with WITH clause (single/multi-line)
- INSERT statements
- UPDATE statements
- DELETE statements
- Comment preservation
- TEXT type preservation
- Edge cases

---

## 🎯 Quick Start

### For Architecture Planning
1. Read: **RUST_PORTING_ANALYSIS.md** sections 1-3
2. Review: **Implementation Priority** (Section 6)
3. Identify: Gotchas & Edge Cases (Section 7)

### For Development
1. Read: **RUST_PORTING_DETAILED.md** sections 1-11
2. Copy: Code examples for your implementation
3. Test: Using guide from **RUST_TEST_MAPPING.md**

### For Testing
1. Run: Manual test execution (RUST_TEST_MAPPING.md section "Manual Test Execution Steps")
2. Check: Test case checklist (RUST_TEST_MAPPING.md)
3. Debug: Using debugging guide (RUST_TEST_MAPPING.md section 5)

---

## 📊 Document Statistics

| Metric | Value |
|---|---|
| Total documentation lines | 2,120 |
| Total documentation size | 61 KB |
| Code examples | 25+ |
| Tables | 30+ |
| Functions documented | 11 major |
| Files analyzed | 3 main |
| Dependencies mapped | 15+ |
| Test cases documented | 8 major |
| Rust crates recommended | 12+ |

---

## 🔑 Key Findings Summary

### Complexity Assessment
- **Overall Complexity**: Medium-High
- **Core Logic**: Medium (straightforward algorithms)
- **Parser Integration**: High (depends on sqlparser-rs)
- **Special Handling**: High (WITH/DISTRIBUTED/PARTITION clauses)
- **HTTP Server**: Low-Medium (standard patterns)

### Critical Components
1. **TEXT Type Restoration** - Non-trivial counting logic
2. **Special Clause Handling** - Complex extraction/restoration
3. **WITH Compression** - Intelligent line width aware logic
4. **Comment Preservation** - Tricky whitespace handling
5. **Parser Integration** - Heavily dependent on sqlparser-rs capabilities

### Estimated Effort
- **Core Library**: 40-60 hours
- **SQL Parser Integration**: 30-50 hours
- **CLI Module**: 20-30 hours
- **HTTP Server**: 30-40 hours
- **Testing & QA**: 40-60 hours
- **Documentation**: 20-30 hours
- **Total**: **200-400 developer-hours**

### High-Risk Areas
1. sqlparser-rs API differences vs CockroachDB parser
2. Pretty-printer algorithm implementation
3. Statement splitting behavior differences
4. Cache eviction semantics
5. TEXT→STRING restoration accuracy with mixed types

---

## 🏗️ Proposed Rust Port Structure

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
│   ├─ args.rs               # Argument parsing (clap)
│   └─ main.rs               # Entry point
│
├─ server/
│   ├─ mod.rs                # HTTP server module
│   ├─ handlers.rs           # Endpoint handlers
│   ├─ cache.rs              # LRU cache implementation
│   └─ templates.rs          # HTML templates
│
└─ wasm/
    └─ lib.rs                # WASM bindings (wasm-pack)
```

---

## 🛠️ Recommended Dependencies

### Core Functionality
```toml
[dependencies]
regex = "1.10"              # Pattern matching (patterns.rs)
once_cell = "1.19"          # Lazy statics (patterns.rs)
sqlparser = "0.45"          # SQL parsing (sqlparser-rs)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # JSON handling (fmt_json)
```

### CLI
```toml
clap = { version = "4.4", features = ["derive"] }  # Argument parsing
anyhow = "1.0"              # Error handling
```

### HTTP Server
```toml
axum = "0.7"                # HTTP framework
tokio = { version = "1", features = ["full"] }
moka = { version = "0.12", features = ["future"] }  # LRU cache
tera = "1.19"               # Template engine
tower = "0.4"               # Middleware
tower-http = { version = "0.5", features = ["cors"] }
```

### WASM
```toml
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = "0.3"
```

### Development
```toml
[dev-dependencies]
assert_cmd = "2.0"          # Integration testing
predicates = "3.1"          # Assertions
```

---

## 📋 Implementation Checklist

### Phase 1: Core Library
- [ ] Create src/lib.rs with public API
- [ ] Implement src/patterns.rs with regex compilation
- [ ] Implement src/text_type.rs
- [ ] Implement src/special_clauses.rs (extraction)
- [ ] Implement special clause restoration
- [ ] Implement comment handling
- [ ] Implement JSON formatting
- [ ] Run basic unit tests

### Phase 2: SQL Parser Integration
- [ ] Evaluate sqlparser-rs capabilities
- [ ] Create src/config.rs for PrettyCfg
- [ ] Implement src/pretty.rs (pretty-printer)
- [ ] Integrate parser with FmtSQL
- [ ] Handle parser error cases
- [ ] Test with simple SQL

### Phase 3: CLI
- [ ] Implement src/cli/args.rs
- [ ] Implement src/cli/main.rs
- [ ] Add stdin handling
- [ ] Add command validation
- [ ] Test with --help and --version
- [ ] Test with test-distributed-by.sql

### Phase 4: HTTP Server
- [ ] Implement src/server/mod.rs
- [ ] Implement src/server/handlers.rs
- [ ] Implement src/server/cache.rs
- [ ] Add template rendering
- [ ] Add error handling
- [ ] Test endpoints with curl

### Phase 5: Testing & Polish
- [ ] Run against test-distributed-by
- [ ] Fix any failures
- [ ] Create additional test cases
- [ ] Performance benchmarking
- [ ] Documentation
- [ ] Release build optimization

### Phase 6: WASM (Optional)
- [ ] Setup wasm-pack
- [ ] Implement src/wasm/lib.rs
- [ ] Build WASM binary
- [ ] Integration with docs/

---

## ⚠️ Critical Gotchas

### 1. TEXT Type Restoration
**Problem**: Simple counting may fail if input has mixed TEXT/STRING types
**Solution**: Track positions instead of counts (low priority fix)

### 2. Parser API Differences
**Problem**: sqlparser-rs may have different statement splitting semantics
**Solution**: Carefully test multi-statement inputs during integration

### 3. Closure Captures in Regex Replacements
**Problem**: Go's closures capture mutably; Rust doesn't allow this in replace_all
**Solution**: Use manual iteration pattern (see Section 4 of DETAILED guide)

### 4. WITH Clause Compression Heuristic
**Problem**: Current logic only checks if compressed fits in linewidth
**Solution**: Consider refining for edge cases; current approach often works

### 5. LRU Cache Eviction
**Problem**: Go implementation clears entire cache when full (not true LRU)
**Solution**: Use moka crate for proper LRU; migrate to true LRU if needed

### 6. Case Mode Function Pointers
**Problem**: Can't store function pointers for closures in Rust
**Solution**: Use enum-based dispatch or function registry

---

## 🧪 Testing Strategy

### Unit Tests
- Text type restoration
- Regex patterns
- Clause extraction
- Compression logic

### Integration Tests
- Full FmtSQL pipeline
- Comment preservation
- Special clause restoration
- CLI argument parsing

### End-to-End Tests
- Run against test-distributed-by.sql
- Compare output byte-for-byte
- Manual inspection of complex cases

### Performance Tests
- Benchmark formatting speed
- Compare to Go version
- Cache hit rates

---

## 📖 Code Example Quick Reference

### TEXT Type Restoration
See: RUST_PORTING_DETAILED.md, Section 4

### Special Clause Extraction
See: RUST_PORTING_DETAILED.md, Section 2

### Regex Patterns
See: RUST_PORTING_DETAILED.md, Section 5

### CLI Argument Mapping
See: RUST_PORTING_DETAILED.md, Section 7

### HTTP Caching
See: RUST_PORTING_DETAILED.md, Section 10

---

## 🔗 External Resources

### sqlparser-rs
- Repository: https://github.com/sqlparser-rs/sqlparser-rs
- Dialect support: PostgreSQL, MySQL, T-SQL, etc.
- Documentation: Check crate docs for AST structure

### Rust Web Frameworks
- **axum**: Modern async web framework (recommended)
- **actix-web**: Alternative (more battle-tested)
- **rocket**: Another option (less control over async)

### Pretty Printer Algorithm
- Original paper: http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf
- Go reference: github.com/cockroachdb/cockroachdb-parser/pkg/util/pretty
- Rust implementations: Check crates.io for "pretty-printer"

---

## 📞 Questions & Answers

### Q: Should we use sqlparser or sqlparser-rs?
**A**: sqlparser-rs is the Rust equivalent. It's well-maintained and supports PostgreSQL.

### Q: What about the pretty-printer algorithm?
**A**: Must implement from scratch based on the algorithm. The Go code uses CockroachDB's implementation which is not available in Rust.

### Q: Can we reuse the test files?
**A**: Yes! test-distributed-by.sql can be used directly. The binary has the same interface.

### Q: What about the WASM build?
**A**: Port core library first, then add WASM bindings with wasm-pack. This is the last phase.

### Q: How long will this take?
**A**: 200-400 developer-hours depending on complexity and team familiarity with Rust.

---

## 📝 Document Maintenance

These documents should be updated if:
- [ ] sqlparser-rs API changes significantly
- [ ] New test cases are added
- [ ] New gotchas are discovered
- [ ] Implementation choices change
- [ ] Performance characteristics change

---

## 🎓 Learning Path

### For New Rust Developers
1. Read RUST_PORTING_ANALYSIS.md for context
2. Study the Go code (sqlfmt.go, backend/main.go)
3. Review RUST_PORTING_DETAILED.md patterns
4. Start with core library (Phase 1)
5. Gradually move to more complex modules

### For Experienced Rust Developers
1. Read RUST_PORTING_DETAILED.md
2. Review RUST_PORTING_ANALYSIS.md gotchas
3. Identify pattern differences
4. Implement systematically by phase

---

## 📌 Key Statistics

| Aspect | Value |
|---|---|
| Go code lines (sqlfmt.go) | 376 |
| Go code lines (backend/main.go) | 827 |
| Total Go code | 1,203 |
| Estimated Rust code | 1,500-2,000 |
| Regex patterns | 5 |
| Main functions | 2 (FmtSQL, FmtJSON) |
| CLI flags | 9 |
| HTTP endpoints | 4 |
| Test files | 2 |
| Documented edge cases | 6+ |
| Recommended Rust crates | 15+ |

---

## 🏁 Getting Started Immediately

1. **Clone and setup**:
   ```bash
   cargo init sqlfmt-rust
   cd sqlfmt-rust
   ```

2. **Add dependencies** (copy from this guide)

3. **Create module structure** (copy from this guide)

4. **Start with Phase 1** (Core Library):
   - Implement patterns.rs first (regex patterns)
   - Then text_type.rs (TEXT restoration)
   - Then special_clauses.rs
   - Finally core.rs (FmtSQL)

5. **Test continuously**:
   ```bash
   cargo test
   ```

6. **Once core works**, test against test-distributed-by:
   ```bash
   cargo build --release
   ./target/release/sqlfmt < ../sqlfmt/tests/test-distributed-by.sql | \
     diff - ../sqlfmt/tests/test-distributed-by.expected
   ```

---

**End of Index. For detailed information, see the individual documentation files.**

Last Updated: May 22, 2026
