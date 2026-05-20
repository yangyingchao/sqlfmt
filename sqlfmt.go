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

// restoreDistributedBy appends DISTRIBUTED BY clauses back to CREATE TABLE statements
// and merges lines if they fit within lineWidth
func restoreDistributedBy(formatted string, distributedClauses []string, lineWidth int) string {
	if len(distributedClauses) == 0 {
		return formatted
	}

	result := formatted
	clauseIdx := 0

	// Find all CREATE TABLE statements and restore the corresponding DISTRIBUTED BY clauses
	createTablePattern := regexp.MustCompile(`(?i)(CREATE\s+TABLE\s+[^;]+?);`)
	result = createTablePattern.ReplaceAllStringFunc(result, func(match string) string {
		if clauseIdx >= len(distributedClauses) {
			return match
		}
		clause := distributedClauses[clauseIdx]
		clauseIdx++
		// Remove the trailing semicolon, add DISTRIBUTED BY, then re-add semicolon
		withClause := strings.TrimSuffix(match, ";") + clause + ";"
		return withClause
	})
	
	// Now handle line merging for lines that have distributed by
	lines := strings.Split(result, "\n")
	var mergedLines []string
	
	for i := 0; i < len(lines); i++ {
		currentLine := lines[i]
		currentLower := strings.ToLower(currentLine)
		
		// Check if current line contains "distributed by" and ends with semicolon
		if strings.Contains(currentLower, "distributed by") && strings.HasSuffix(strings.TrimSpace(currentLine), ";") {
			// Try to merge with previous line if it exists and previous line doesn't end with semicolon
			if i > 0 && len(mergedLines) > 0 {
				prevLine := mergedLines[len(mergedLines)-1]
				prevLower := strings.ToLower(prevLine)
				
				// Only merge if previous line doesn't already contain "distributed by"
				if !strings.Contains(prevLower, "distributed by") {
					prevTrimmed := strings.TrimSpace(prevLine)
					currentTrimmed := strings.TrimSpace(currentLine)
					
					// Calculate merged length
					mergedLength := len(prevTrimmed) + 1 + len(currentTrimmed)
					
					if mergedLength <= lineWidth {
						// Merge: remove the previous line, add merged version
						mergedLines = mergedLines[:len(mergedLines)-1]
						mergedLines = append(mergedLines, prevTrimmed+" "+currentTrimmed)
						continue
					}
				}
			}
		}
		
		mergedLines = append(mergedLines, currentLine)
	}
	
	return strings.Join(mergedLines, "\n")
}

func FmtSQL(cfg tree.PrettyCfg, stmts []string) (string, error) {
	var prettied strings.Builder
	var allDistributedClauses []string
	totalTextCount := 0

	for _, stmt := range stmts {
		// Strip TEXT type to prevent CockroachDB parser from converting it to STRING
		stmt, textCount := stripTextType(stmt)
		totalTextCount += textCount
		
		// Strip DISTRIBUTED BY clauses before processing
		stmt, distributedClauses := stripDistributedBy(stmt)
		allDistributedClauses = append(allDistributedClauses, distributedClauses...)

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
	// Restore DISTRIBUTED BY clauses
	result = restoreDistributedBy(result, allDistributedClauses, cfg.LineWidth)
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
