use crate::fingerprint::fingerprint;
use crate::issues::ExistingIssue;
use crate::scan::Found;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize, Debug, PartialEq)]
pub struct Create {
    pub title: String,
    pub body: String,
    pub fingerprint: String,
    pub file: String,
    pub line: usize,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Close {
    pub number: u64,
    pub fingerprint: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Actions {
    pub create: Vec<Create>,
    pub close: Vec<Close>,
}

pub fn render_body(found: &Found, fp: &str) -> String {
    let mut b = String::new();
    if !found.marker.body.is_empty() {
        b.push_str(&found.marker.body);
        b.push_str("\n\n");
    }
    if !found.context.is_empty() {
        b.push_str(&format!("```{}\n{}\n```\n\n", found.lang, found.context));
    }
    b.push_str(&format!(
        "---\nFound at `{}:{}`\n\n<!-- 0jot: {} -->\n",
        found.file, found.marker.line, fp
    ));
    b
}

pub fn plan(found: &[Found], existing: &[ExistingIssue]) -> Actions {
    let existing_fps: BTreeSet<&str> = existing.iter().map(|e| e.fingerprint.as_str()).collect();
    let mut code_fps: BTreeSet<String> = BTreeSet::new();
    let mut create = Vec::new();
    for f in found {
        let fp = fingerprint(&f.marker.title);
        if !code_fps.insert(fp.clone()) {
            continue; // duplicate title already accounted for
        }
        if !existing_fps.contains(fp.as_str()) {
            create.push(Create {
                title: f.marker.title.clone(),
                body: render_body(f, &fp),
                fingerprint: fp,
                file: f.file.clone(),
                line: f.marker.line,
            });
        }
    }
    let close = existing
        .iter()
        .filter(|e| !code_fps.contains(&e.fingerprint))
        .map(|e| Close {
            number: e.number,
            fingerprint: e.fingerprint.clone(),
        })
        .collect();
    Actions { create, close }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::fingerprint;
    use crate::issues::ExistingIssue;
    use crate::marker::Marker;
    use crate::scan::Found;

    fn found(title: &str) -> Found {
        Found {
            marker: Marker {
                title: title.into(),
                body: String::new(),
                line: 1,
            },
            file: "src/x.rs".into(),
            context: String::new(),
            lang: "rs".into(),
        }
    }

    #[test]
    fn create_for_new_marker() {
        let f = vec![found("alpha")];
        let actions = plan(&f, &[]);
        assert_eq!(actions.create.len(), 1);
        assert_eq!(actions.create[0].title, "alpha");
        assert_eq!(actions.create[0].fingerprint, fingerprint("alpha"));
        assert!(actions.close.is_empty());
    }

    #[test]
    fn close_for_removed_marker() {
        let existing = vec![ExistingIssue {
            number: 7,
            fingerprint: fingerprint("gone"),
        }];
        let actions = plan(&[], &existing);
        assert_eq!(
            actions.close,
            vec![Close {
                number: 7,
                fingerprint: fingerprint("gone")
            }]
        );
        assert!(actions.create.is_empty());
    }

    #[test]
    fn noop_when_marker_matches_issue() {
        let f = vec![found("same")];
        let existing = vec![ExistingIssue {
            number: 1,
            fingerprint: fingerprint("same"),
        }];
        let actions = plan(&f, &existing);
        assert!(actions.create.is_empty());
        assert!(actions.close.is_empty());
    }

    #[test]
    fn duplicate_titles_create_once() {
        let f = vec![found("dup"), found("dup")];
        let actions = plan(&f, &[]);
        assert_eq!(actions.create.len(), 1);
    }

    #[test]
    fn body_has_location_and_fingerprint_comment() {
        let mut fnd = found("t");
        fnd.marker.body = "line one".into();
        fnd.marker.line = 42;
        fnd.context = "let a = 1;\nlet b = 2;".into();
        let body = render_body(&fnd, "abcdef012345");
        assert!(body.contains("line one"));
        assert!(body.contains("```rs"));
        assert!(body.contains("let a = 1;"));
        assert!(body.contains("Found at `src/x.rs:42`"));
        assert!(body.contains("<!-- 0jot: abcdef012345 -->"));
    }
}
