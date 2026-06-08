use std::path::{Component, Path, PathBuf};

/// Join `rel` onto `root` and return the canonical path only if it stays
/// inside `root`. Returns `None` on any escape attempt or missing target.
///
/// Used for read paths where the target must already exist (so we can
/// canonicalize and compare against the canonical root).
pub fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let candidate = root.join(rel);
    let canon = candidate.canonicalize().ok()?;
    let canon_root = root.canonicalize().ok()?;
    canon.starts_with(&canon_root).then_some(canon)
}

/// Reduce an upload filename to a safe basename: strip any directory
/// components and reject `.`/`..`. Returns "" if nothing safe remains.
pub fn sanitize_filename(name: &str) -> String {
    let last = Path::new(name)
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .next_back();
    match last {
        Some(s) if s != "." && s != ".." => s.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolves_child_inside_root() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"hi").unwrap();
        let p = safe_join(dir.path(), "a.txt").unwrap();
        assert!(p.ends_with("a.txt"));
    }

    #[test]
    fn rejects_parent_escape() {
        let dir = tempdir().unwrap();
        assert!(safe_join(dir.path(), "../etc/passwd").is_none());
    }

    #[test]
    fn rejects_absolute() {
        let dir = tempdir().unwrap();
        assert!(safe_join(dir.path(), "/etc/passwd").is_none());
    }

    #[test]
    fn sanitizes_upload_filename_to_basename() {
        assert_eq!(sanitize_filename("../../evil.sh"), "evil.sh");
        assert_eq!(sanitize_filename("plain.txt"), "plain.txt");
        assert_eq!(sanitize_filename("a/b/c.bin"), "c.bin");
    }

    #[test]
    fn rejects_empty_or_dotdot_filename() {
        assert_eq!(sanitize_filename(".."), "");
        assert_eq!(sanitize_filename(""), "");
    }
}
