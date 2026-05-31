#[derive(Debug, PartialEq, Clone)]
pub struct Marker {
    pub title: String,
    pub body: String,
    pub line: usize,
}

const PREFIXES: [&str; 6] = ["//", "/*", "#", "*", ";", "--"];

// @todo: support block-comment body capture
//   Markers inside `/* ... */` blocks only capture the title line; body lines
//   at the same indent are not collected. Extend the parser to handle them.

/// Returns the text following a leading comment prefix, or None if the line is
/// not a comment. The returned slice keeps the comment's internal indentation.
fn comment_rest(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    for p in PREFIXES {
        if let Some(rest) = trimmed.strip_prefix(p) {
            return Some(rest);
        }
    }
    None
}

fn indent(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

pub fn parse(text: &str) -> Vec<Marker> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // --- block comment marker: line starts with `/*` and contains `@todo:` ---
        if let Some(rest) = lines[i].trim_start().strip_prefix("/*") {
            if let Some(after) = rest.trim_start().strip_prefix("@todo:") {
                let (title_part, closed_here) = match after.find("*/") {
                    Some(pos) => (&after[..pos], true),
                    None => (after, false),
                };
                let title = title_part.trim().to_string();
                if !title.is_empty() {
                    let mut body_lines = Vec::new();
                    let mut j = i + 1;
                    if !closed_here {
                        while j < lines.len() {
                            let line = lines[j];
                            let (content, closing) = match line.find("*/") {
                                Some(pos) => (&line[..pos], true),
                                None => (line, false),
                            };
                            let c = content.trim_start();
                            let c = c.strip_prefix('*').unwrap_or(c).trim();
                            if !c.is_empty() {
                                body_lines.push(c.to_string());
                            }
                            j += 1;
                            if closing {
                                break;
                            }
                        }
                    }
                    out.push(Marker {
                        title,
                        body: body_lines.join("\n"),
                        line: i + 1,
                    });
                    i = j;
                    continue;
                }
            }
        }

        // --- line comment marker (existing behavior) ---
        let Some(rest) = comment_rest(lines[i]) else {
            i += 1;
            continue;
        };
        let Some(after) = rest.trim_start().strip_prefix("@todo:") else {
            i += 1;
            continue;
        };
        let title = after.trim().to_string();
        if title.is_empty() {
            i += 1;
            continue;
        }
        let marker_indent = indent(rest);
        let mut body_lines = Vec::new();
        let mut j = i + 1;
        while j < lines.len() {
            match comment_rest(lines[j]) {
                Some(r) if indent(r) > marker_indent => {
                    body_lines.push(r.trim().to_string());
                    j += 1;
                }
                _ => break,
            }
        }
        out.push(Marker {
            title,
            body: body_lines.join("\n"),
            line: i + 1,
        });
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_only() {
        let m = parse("// @todo: do the thing\n");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "do the thing");
        assert_eq!(m[0].body, "");
        assert_eq!(m[0].line, 1);
    }

    #[test]
    fn title_with_body() {
        let src = "fn x() {}\n// @todo: fix x\n//   it is broken\n//   really broken\nlet y = 1;\n";
        let m = parse(src);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "fix x");
        assert_eq!(m[0].body, "it is broken\nreally broken");
        assert_eq!(m[0].line, 2);
    }

    #[test]
    fn plain_following_comment_is_not_body() {
        let src = "// @todo: a\n// not indented, not body\n";
        let m = parse(src);
        assert_eq!(m[0].body, "");
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn empty_title_ignored() {
        let m = parse("// @todo:   \n");
        assert!(m.is_empty());
    }

    #[test]
    fn hash_prefix_marker() {
        let m = parse("# @todo: shell todo\n");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "shell todo");
    }

    #[test]
    fn multiple_markers() {
        let m = parse("// @todo: one\ncode\n// @todo: two\n");
        assert_eq!(m.len(), 2);
        assert_eq!(m[1].title, "two");
        assert_eq!(m[1].line, 3);
    }

    #[test]
    fn block_comment_marker_with_body() {
        let src = "/* @todo: block title\n * body one\n * body two\n */\n";
        let m = parse(src);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "block title");
        assert_eq!(m[0].body, "body one\nbody two");
        assert_eq!(m[0].line, 1);
    }

    #[test]
    fn single_line_block_marker() {
        let m = parse("/* @todo: one liner */\n");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "one liner");
        assert_eq!(m[0].body, "");
    }

    #[test]
    fn block_marker_title_strips_trailing_close() {
        // closing on the title line, no body
        let m = parse("/* @todo: tight */\ncode\n");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].title, "tight");
        assert_eq!(m[0].body, "");
    }
}
