use serde::Deserialize;

#[derive(Deserialize)]
struct RawIssue {
    number: u64,
    body: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ExistingIssue {
    pub number: u64,
    pub fingerprint: String,
}

pub fn extract_fp(body: &str) -> Option<String> {
    let start = body.find("<!-- 0jot:")?;
    let rest = &body[start + "<!-- 0jot:".len()..];
    let end = rest.find("-->")?;
    Some(rest[..end].trim().to_string())
}

pub fn parse(json: &str) -> Vec<ExistingIssue> {
    let raw: Vec<RawIssue> = serde_json::from_str(json).unwrap_or_default();
    raw.into_iter()
        .filter_map(|i| {
            extract_fp(i.body.as_deref().unwrap_or("")).map(|fp| ExistingIssue {
                number: i.number,
                fingerprint: fp,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_fingerprint_from_body() {
        let body = "blah\n\n<!-- 0jot: abc123def456 -->\n";
        assert_eq!(extract_fp(body), Some("abc123def456".to_string()));
    }

    #[test]
    fn none_when_no_marker() {
        assert_eq!(extract_fp("just text"), None);
    }

    #[test]
    fn parse_filters_issues_without_marker() {
        let json = r#"[
            {"number": 1, "body": "x <!-- 0jot: aaaaaaaaaaaa --> y"},
            {"number": 2, "body": "no marker here"},
            {"number": 3, "body": null}
        ]"#;
        let issues = parse(json);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].number, 1);
        assert_eq!(issues[0].fingerprint, "aaaaaaaaaaaa");
    }
}
