/// Width-aware SQL formatting.
/// Preserves original spacing from sqlparser. Breaks at depth-0 keywords
/// and commas. Each keyword gets its own line; content is indented +1.
use crate::config::FormatterConfig;

const KW: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "INTO",
    "VALUES",
    "SET",
    "ORDER",
    "BY",
    "GROUP",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "JOIN",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "CROSS",
    "FULL",
    "NATURAL",
    "USING",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "RETURNING",
    "CASE",
    "INSERT",
    "UPDATE",
    "DELETE",
    "CREATE",
    "DROP",
    "ALTER",
    "TABLE",
    "DISTRIBUTED",
    "PARTITION",
];

pub fn apply_width(sql: &str, cfg: &FormatterConfig) -> String {
    let indent = cfg.indent_str();
    let mut r = String::new();
    for line in sql.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("--") {
            r.push_str(line);
            r.push('\n');
            continue;
        }
        if t.len() <= cfg.print_width {
            r.push_str(line);
            r.push('\n');
            continue;
        }
        let has_semi = t.ends_with(';');
        let core = if has_semi { &t[..t.len() - 1] } else { t };
        r.push_str(&fmt(core, cfg.print_width, &indent, cfg.indent_width, 0));
        if has_semi {
            r.push(';');
        }
        r.push('\n');
    }
    r
}

fn fmt(sql: &str, margin: usize, indent: &str, tw: usize, lvl: usize) -> String {
    if lvl > 10 {
        let ind = indent.repeat(lvl);
        return if lvl == 0 {
            sql.to_string()
        } else {
            format!("{}{}", ind, sql)
        };
    }
    let used = lvl * if indent == "\t" { tw } else { indent.len() };
    let avail = margin.saturating_sub(used);
    if sql.len() <= avail {
        let ind = indent.repeat(lvl);
        return if lvl == 0 {
            sql.to_string()
        } else {
            format!("{}{}", ind, sql)
        };
    }

    let bs = find_breaks(sql);
    if bs.is_empty() {
        let ind = indent.repeat(lvl);
        return if lvl == 0 {
            sql.to_string()
        } else {
            format!("{}{}", ind, sql)
        };
    }

    let ci = indent.repeat(lvl);
    let ni = indent.repeat(lvl + 1);
    let mut out = String::new();
    let n = sql.len();

    // Segment 0: before first break
    let seg0 = sql[0..bs[0].0].trim();
    if !seg0.is_empty() {
        out.push_str(&ci);
        out.push_str(seg0);
    }

    // Each segment starting at bs[i].pos, ending at bs[i+1].pos or EOF
    for i in 0..bs.len() {
        let start = bs[i].0;
        let end = if i + 1 < bs.len() { bs[i + 1].0 } else { n };
        let kind = bs[i].1; // 0=keyword, 1=comma
        let seg = sql[start..end].trim();
        if seg.is_empty() {
            continue;
        }

        if kind == 0 {
            // Keyword: split into keyword word + rest
            let sp = seg.find(char::is_whitespace).unwrap_or(seg.len());
            let kw_w = &seg[..sp];
            let rest = seg[sp..].trim();
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&ci);
            out.push_str(kw_w);
            if !rest.is_empty() {
                out.push('\n');
                out.push_str(&ni);
                reemit(&mut out, rest, margin, indent, tw, lvl + 1);
            }
        } else {
            // Comma: output content at ni (comma is in previous segment)
            let trimmed = seg.trim();
            if !trimmed.is_empty() {
                out.push('\n');
                out.push_str(&ni);
                reemit(&mut out, trimmed, margin, indent, tw, lvl + 1);
            }
        }
    }

    if lvl == 0 {
        if let Some(s) = out.strip_prefix(indent) {
            return s.to_string();
        }
    }
    out
}

fn find_breaks(sql: &str) -> Vec<(usize, u8)> {
    let chars: Vec<char> = sql.chars().collect();
    // Build char-index → byte-index lookup
    let c2b: Vec<usize> = {
        let mut v = Vec::with_capacity(chars.len() + 1);
        let mut bp = 0;
        for &c in &chars {
            v.push(bp);
            bp += c.len_utf8();
        }
        v.push(bp);
        v
    };
    let cn = chars.len();

    let mut bs: Vec<(usize, u8)> = Vec::new();
    let mut i = 0;
    let mut depth: i32 = 0;
    let mut ins = false;
    let mut sc = ' ';

    while i < cn {
        let ch = chars[i];
        if ins {
            if ch == sc {
                ins = false;
            }
            i += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            ins = true;
            sc = ch;
            i += 1;
            continue;
        }
        if ch == '(' {
            depth += 1;
            i += 1;
            continue;
        }
        if ch == ')' {
            depth -= 1;
            i += 1;
            continue;
        }
        if ch == ',' && depth == 0 {
            bs.push((c2b[i + 1], 1));
            i += 1;
            continue;
        }
        if ch.is_whitespace() && depth == 0 && i > 0 {
            let mut ws = i;
            while ws > 0
                && !chars[ws - 1].is_whitespace()
                && chars[ws - 1] != '('
                && chars[ws - 1] != ')'
                && chars[ws - 1] != ','
            {
                ws -= 1;
            }
            if ws < i {
                let w: String = chars[ws..i].iter().collect();
                let u = w.to_uppercase();
                if is_kw(&u) {
                    if u == "BY" {
                        let before = &sql[..c2b[ws]].trim();
                        let last = before.split_whitespace().last().unwrap_or("");
                        if let Some(ul) = last.split_whitespace().last() {
                            let ul = ul.to_uppercase();
                            if ul == "ORDER"
                                || ul == "GROUP"
                                || ul == "DISTRIBUTED"
                                || ul == "PARTITION"
                            {
                                i += 1;
                                continue;
                            }
                        }
                    }
                    bs.push((c2b[ws], 0));
                }
            }
        }
        i += 1;
    }

    if depth == 0 {
        let mut ws = cn;
        while ws > 0
            && !chars[ws - 1].is_whitespace()
            && chars[ws - 1] != '('
            && chars[ws - 1] != ')'
            && chars[ws - 1] != ','
        {
            ws -= 1;
        }
        if ws < cn {
            let w: String = chars[ws..cn].iter().collect();
            let u = w.to_uppercase();
            if is_kw(&u) {
                bs.push((c2b[ws], 0));
            }
        }
    }

    bs.sort_by_key(|&(p, _)| p);
    bs.dedup_by_key(|&mut (p, _)| p);
    bs
}

fn reemit(out: &mut String, text: &str, margin: usize, indent: &str, tw: usize, lvl: usize) {
    let used = lvl * if indent == "\t" { tw } else { indent.len() };
    let avail = margin.saturating_sub(used);
    if text.len() <= avail {
        out.push_str(text);
    } else {
        let rec = fmt(text, margin, indent, tw, lvl);
        out.push_str(rec.trim_start());
    }
}

fn is_kw(s: &str) -> bool {
    KW.contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FormatterConfig;
    #[test]
    fn test_short() {
        let r = apply_width("SELECT 1", &FormatterConfig::default());
        assert_eq!(r.trim(), "SELECT 1");
    }
}
