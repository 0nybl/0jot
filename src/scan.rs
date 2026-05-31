use crate::ignore::Ignore;
use crate::marker::{self, Marker};
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct Found {
    pub marker: Marker,
    pub file: String,
}

const SKIP_DIRS: [&str; 3] = [".git", "target", "node_modules"];

pub fn scan(root: &Path) -> Vec<Found> {
    let ignore = Ignore::load(root);
    let mut out = Vec::new();
    walk(root, root, &ignore, &mut out);
    out.sort_by(|a, b| (&a.file, a.marker.line).cmp(&(&b.file, b.marker.line)));
    out
}

fn walk(root: &Path, dir: &Path, ignore: &Ignore, out: &mut Vec<Found>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk(root, &path, ignore, out);
        } else {
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let text = match String::from_utf8(bytes) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if ignore.is_ignored(&rel) {
                continue;
            }
            for m in marker::parse(&text) {
                out.push(Found {
                    marker: m,
                    file: rel.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(dir: &std::path::Path, rel: &str, body: &[u8]) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    #[test]
    fn respects_0jotignore() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "src/a.rs", b"// @todo: keep\n");
        write(root.path(), "README.md", b"// @todo: drop me\n");
        write(root.path(), ".0jotignore", b"*.md\n");
        let found = scan(root.path());
        let titles: Vec<&str> = found.iter().map(|f| f.marker.title.as_str()).collect();
        assert_eq!(titles, vec!["keep"]);
    }

    #[test]
    fn finds_markers_skips_target_and_binary() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "src/a.rs", b"// @todo: alpha\n");
        write(root.path(), "src/b.rs", b"// @todo: beta\n");
        write(root.path(), "target/c.rs", b"// @todo: should be skipped\n");
        write(root.path(), "bin.dat", &[0xff, 0xfe, 0x00, 0x01]); // not utf-8

        let found = scan(root.path());
        let titles: Vec<&str> = found.iter().map(|f| f.marker.title.as_str()).collect();
        assert_eq!(titles, vec!["alpha", "beta"]); // sorted by file, target skipped
        assert_eq!(found[0].file, "src/a.rs");
    }
}
