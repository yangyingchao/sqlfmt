package sqlfmt

import (
	"regexp"
	"strings"
	"unicode"

	"github.com/cockroachdb/cockroachdb-parser/pkg/sql/parser"
	"github.com/cockroachdb/cockroachdb-parser/pkg/sql/sem/tree"
	"github.com/cockroachdb/cockroachdb-parser/pkg/util/json"
	"github.com/cockroachdb/cockroachdb-parser/pkg/util/pretty"
)

var (
	ignoreComments       = regexp.MustCompile(`^--.*\s*`)
	distributedByPattern = regexp.MustCompile(`(?i)\s+distributed\s+by\s*\([^)]*\)`)
	withClausePattern    = regexp.MustCompile(`(?i)\s+with\s*\([^)]*\)`)
	textTypePattern      = regexp.MustCompile(`(?i)\bTEXT\b`)
)

// stripTextType records positions of TEXT types before parsing
// Since CockroachDB parser converts TEXT to STRING, we track how many TEXT occurrences exist
func stripTextType(sql string) (string, int) {
	// Count occurrences of TEXT type (case-insensitive, word boundary)
	matches := textTypePattern.FindAllStringIndex(sql, -1)
	textCount := len(matches)

	// Don't modify the SQL, just count TEXT occurrences
	// The parser will convert TEXT to STRING, and we'll restore it later
	return sql, textCount
}

// restoreTextType restores TEXT type after formatting
// CockroachDB parser converts TEXT to STRING, so we need to restore it
// We replace the first N STRING occurrences with TEXT (where N = number of original TEXT types)
func restoreTextType(formatted string, textCount int) string {
	if textCount == 0 {
		return formatted
	}

	// Replace STRING with TEXT for the first textCount occurrences
	// This assumes that STRING types in the output correspond to the original TEXT types
	stringPattern := regexp.MustCompile(`(?i)\bSTRING\b`)

	// Track how many STRING instances we've seen
	replaced := 0
	result := stringPattern.ReplaceAllStringFunc(formatted, func(match string) string {
		if replaced < textCount {
			replaced++
			// Preserve the case from the match (though STRING is usually uppercase)
			return "TEXT"
		}
		return match
	})

	return result
}

// stripDistributedBy removes the DISTRIBUTED BY clause from SQL statements
// and returns both the cleaned SQL and the extracted clause for later restoration
func stripDistributedBy(sql string) (string, []string) {
	matches := distributedByPattern.FindAllString(sql, -1)
	cleanedSQL := distributedByPattern.ReplaceAllString(sql, "")
	return cleanedSQL, matches
}

// stripWithClause removes the WITH clause from SQL statements (for CREATE TABLE statements)
// and returns both the cleaned SQL and the extracted clause for later restoration
func stripWithClause(sql string) (string, []string) {
	matches := withClausePattern.FindAllString(sql, -1)
	cleanedSQL := withClausePattern.ReplaceAllString(sql, "")
	return cleanedSQL, matches
}

// extractAndStripClauses extracts WITH and DISTRIBUTED BY clauses from CREATE TABLE statements
// and returns the cleaned SQL plus the list of clause sets for each CREATE TABLE
func extractAndStripClauses(sql string, allClauses []map[string][]string) (string, []map[string][]string) {
	result := sql

	// Find all CREATE TABLE statements and extract their clauses
	createTablePattern := regexp.MustCompile(`(?i)CREATE\s+TABLE\s+[^;]+?;`)

	matches := createTablePattern.FindAllString(result, -1)
	for _, match := range matches {
		// Extract WITH clause (both original and compressed versions)
		withMatches := withClausePattern.FindAllString(match, -1)
		withClauseOriginal := ""
		withClauseCompressed := ""
		if len(withMatches) > 0 {
			withClauseOriginal = withMatches[0]
			// Clean up leading whitespace/newlines from original
			withClauseOriginal = regexp.MustCompile(`^\s*`).ReplaceAllString(withClauseOriginal, "")

			// Also create a compressed version for checking if it fits
			withClauseCompressed = withMatches[0]
			withClauseCompressed = regexp.MustCompile(`\s+`).ReplaceAllString(withClauseCompressed, " ")
			withClauseCompressed = strings.TrimSpace(withClauseCompressed)
		}

		// Extract DISTRIBUTED BY clause
		distMatches := distributedByPattern.FindAllString(match, -1)
		distClause := ""
		if len(distMatches) > 0 {
			// Trim leading/trailing whitespace from the extracted clause
			distClause = strings.TrimSpace(distMatches[0])
		}

		clauses := map[string][]string{
			"WITH":              {withClauseCompressed},
			"WITH_ORIGINAL":     {withClauseOriginal},
			"DISTRIBUTED":       {distClause},
		}
		allClauses = append(allClauses, clauses)
	}

	// Remove all clauses from SQL
	result = withClausePattern.ReplaceAllString(result, "")
	result = distributedByPattern.ReplaceAllString(result, "")

	return result, allClauses
}

// mergeLineIfFits checks if merging two lines would exceed lineWidth
// If not, it merges them; otherwise keeps them on separate lines
func mergeLineIfFits(prevLine, currentLine string, lineWidth int) string {
	// Trim trailing newline from prevLine and leading/trailing spaces from currentLine
	prevLineTrimmed := strings.TrimSuffix(prevLine, "\n")
	currentLineTrimmed := strings.TrimSpace(currentLine)

	// Calculate the merged length (with a space between them)
	mergedLength := len(prevLineTrimmed) + 1 + len(currentLineTrimmed)

	if mergedLength <= lineWidth {
		// Fits within line width, merge them
		return prevLineTrimmed + " " + currentLineTrimmed + "\n"
	}
	// Doesn't fit, return as is
	return prevLine + currentLine
}

// restoreAllSpecialClauses restores both WITH and DISTRIBUTED BY clauses
// to CREATE TABLE statements in the correct order, with intelligent line merging
func restoreAllSpecialClauses(formatted string, allClauses []map[string][]string, lineWidth int) string {
	if len(allClauses) == 0 {
		return formatted
	}

	result := formatted
	clauseIdx := 0

	// Find all CREATE TABLE statements and restore the corresponding clauses
	createTablePattern := regexp.MustCompile(`(?i)(CREATE\s+TABLE\s+[^;]+?);`)
	result = createTablePattern.ReplaceAllStringFunc(result, func(match string) string {
		if clauseIdx >= len(allClauses) {
			return match
		}
		clauses := allClauses[clauseIdx]
		clauseIdx++

		// Remove the trailing semicolon
		statement := strings.TrimSuffix(match, ";")

		distClause := ""
		if len(clauses["DISTRIBUTED"]) > 0 && clauses["DISTRIBUTED"][0] != "" {
			distClause = clauses["DISTRIBUTED"][0]
		}

		// Get original and compressed WITH clauses
		withClauseCompressed := ""
		withClauseOriginal := ""
		if len(clauses["WITH"]) > 0 {
			withClauseCompressed = clauses["WITH"][0]
		}
		if len(clauses["WITH_ORIGINAL"]) > 0 {
			withClauseOriginal = clauses["WITH_ORIGINAL"][0]
		}

		// Decide which version of WITH clause to use
		withClause := withClauseCompressed

		// Check if compressed WITH itself fits within lineWidth
		if withClauseCompressed != "" && len(withClauseCompressed) > lineWidth {
			// Compressed WITH is too long, use original multi-line format
			withClause = withClauseOriginal
		}

		// Add WITH clause to statement
		if withClause != "" {
			// Always add newline before WITH, trim leading whitespace
			statement += "\n" + strings.TrimLeft(withClause, " \t")
		}

		// Add DISTRIBUTED BY clause on a new line
		if distClause != "" {
			statement += "\n" + distClause
		}

		statement += ";"
		return statement
	})

	return result
}

func FmtSQL(cfg tree.PrettyCfg, stmts []string) (string, error) {
	var prettied strings.Builder
	var allSpecialClauses []map[string][]string // Store WITH and DISTRIBUTED BY for each CREATE TABLE
	totalTextCount := 0

	for _, stmt := range stmts {
		// Strip TEXT type to prevent CockroachDB parser from converting it to STRING
		stmt, textCount := stripTextType(stmt)
		totalTextCount += textCount

		// Extract all WITH and DISTRIBUTED BY clauses for this statement
		stmt, allSpecialClauses = extractAndStripClauses(stmt, allSpecialClauses)

		for len(stmt) > 0 {
			stmt = strings.TrimSpace(stmt)
			hasContent := false
			// Trim comments, preserving whitespace after them.
			for {
				found := ignoreComments.FindString(stmt)
				if found == "" {
					break
				}
				// Remove trailing whitespace but keep up to 2 newlines.
				prettied.WriteString(strings.TrimRightFunc(found, unicode.IsSpace))
				newlines := strings.Count(found, "\n")
				if newlines > 2 {
					newlines = 2
				}
				prettied.WriteString(strings.Repeat("\n", newlines))
				stmt = stmt[len(found):]
				hasContent = true
			}
			// Split by semicolons
			next := stmt
			if pos, _ := parser.SplitFirstStatement(stmt); pos > 0 {
				next = stmt[:pos]
				stmt = stmt[pos:]
			} else {
				stmt = ""
			}
			// This should only return 0 or 1 responses.
			allParsed, err := parser.Parse(next)
			if err != nil {
				return "", err
			}
			for _, parsed := range allParsed {
				pretty, err := cfg.Pretty(parsed.AST)
				if err != nil {
					return "", err
				}
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
	// Restore TEXT type (CockroachDB parser converts TEXT to STRING)
	result = restoreTextType(result, totalTextCount)
	// Restore WITH and DISTRIBUTED BY clauses in the correct order
	result = restoreAllSpecialClauses(result, allSpecialClauses, cfg.LineWidth)
	return result, nil
}

func FmtJSON(s string) (pretty.Doc, error) {
	j, err := json.ParseJSON(s)
	if err != nil {
		return nil, err
	}
	return fmtJSONNode(j), nil
}

func fmtJSONNode(j json.JSON) pretty.Doc {
	// Figure out what type this is.
	if it, _ := j.ObjectIter(); it != nil {
		// Object.
		elems := make([]pretty.Doc, 0, j.Len())
		for it.Next() {
			elems = append(elems, pretty.NestUnder(
				pretty.Concat(
					pretty.Text(json.FromString(it.Key()).String()),
					pretty.Text(`:`),
				),
				fmtJSONNode(it.Value()),
			))
		}
		return prettyBracket("{", elems, "}")
	} else if n := j.Len(); n > 0 {
		// Non-empty array.
		elems := make([]pretty.Doc, n)
		for i := 0; i < n; i++ {
			elem, err := j.FetchValIdx(i)
			if err != nil {
				return pretty.Text(j.String())
			}
			elems[i] = fmtJSONNode(elem)
		}
		return prettyBracket("[", elems, "]")
	}
	// Other.
	return pretty.Text(j.String())
}

func prettyBracket(l string, elems []pretty.Doc, r string) pretty.Doc {
	return pretty.BracketDoc(pretty.Text(l), pretty.Join(",", elems...), pretty.Text(r))
}
