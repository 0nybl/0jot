use glob::Pattern;
use std::path::Path;

pub struct Ignore {
    rules: Vec<Rule>,
}

enum Rule {
    /// Directory rule from a trailing-slash pattern.
    Dir { name: String, has_slash: bool },
    /// File glob; `basename_only` when the pattern had no `/`.
    File { glob: Pattern, basename_only: bool },
}

impl Ignore {
    pub fn parse(text: &str) -> Ignore {
        let mut rules = Vec::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(dir) = line.strip_suffix('/') {
                let dir = dir.trim_start_matches('/');
                rules.push(Rule::Dir {
                    name: dir.to_string(),
                    has_slash: dir.contains('/'),
                });
            } else {
                let p = line.trim_start_matches('/');
                if let Ok(glob) = Pattern::new(p) {
                    rules.push(Rule::File {
                        glob,
                        basename_only: !p.contains('/'),
                    });
                }
            }
        }
        Ignore { rules }
    }

    pub fn load(root: &Path) -> Ignore {
        let text = std::fs::read_to_string(root.join(".0jotignore")).unwrap_or_default();
        Ignore::parse(&text)
    }

    pub fn is_ignored(&self, rel: &str) -> bool {
        for rule in &self.rules {
            match rule {
                Rule::Dir { name, has_slash } => {
                    if *has_slash {
                        if rel == name || rel.starts_with(&format!("{name}/")) {
                            return true;
                        }
                    } else if rel.split('/').any(|seg| seg == name) {
                        return true;
                    }
                }
                Rule::File {
                    glob,
                    basename_only,
                } => {
                    if *basename_only {
                        let base = rel.rsplit('/').next().unwrap_or(rel);
                        if glob.matches(base) {
                            return true;
                        }
                    } else if glob.matches(rel) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_glob_matches_anywhere() {
        let ig = Ignore::parse("*.md\n");
        assert!(ig.is_ignored("README.md"));
        assert!(ig.is_ignored("docs/spec.md"));
        assert!(!ig.is_ignored("src/a.rs"));
    }

    #[test]
    fn dir_rule_matches_segment() {
        let ig = Ignore::parse("docs/\n");
        assert!(ig.is_ignored("docs/spec.md"));
        assert!(ig.is_ignored("a/docs/b.rs"));
        assert!(!ig.is_ignored("src/a.rs"));
    }

    #[test]
    fn slash_pattern_matches_full_path() {
        let ig = Ignore::parse("src/gen/*.rs\n");
        assert!(ig.is_ignored("src/gen/x.rs"));
        assert!(!ig.is_ignored("src/x.rs"));
    }

    #[test]
    fn comments_and_blanks_ignored() {
        let ig = Ignore::parse("# a comment\n\n*.md\n");
        assert!(ig.is_ignored("x.md"));
        assert!(!ig.is_ignored("x.rs"));
    }

    #[test]
    fn empty_ignore_matches_nothing() {
        let ig = Ignore::parse("");
        assert!(!ig.is_ignored("anything.md"));
    }
}
