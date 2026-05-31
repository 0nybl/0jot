use sha2::{Digest, Sha256};

pub fn normalize_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn fingerprint(title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalize_title(title).as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_is_normalized() {
        assert_eq!(
            fingerprint("fix   the\tthing"),
            fingerprint("fix the thing")
        );
    }

    #[test]
    fn different_titles_differ() {
        assert_ne!(fingerprint("a"), fingerprint("b"));
    }

    #[test]
    fn is_twelve_hex_chars() {
        let fp = fingerprint("anything");
        assert_eq!(fp.len(), 12);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
