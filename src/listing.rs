use humansize::{DECIMAL, format_size};
use std::fs;
use std::path::Path;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const TEMPLATE: &str = include_str!("templates/listing.html");

/// Render an HTML directory index for `dir`. `url_path` is the request path
/// (used for the breadcrumb heading). `upload` controls the upload widget.
pub fn render_listing(dir: &Path, url_path: &str, upload: bool) -> std::io::Result<String> {
    let mut entries: Vec<(String, bool, u64, Option<OffsetDateTime>)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let modified = meta.modified().ok().map(OffsetDateTime::from);
        entries.push((name, meta.is_dir(), meta.len(), modified));
    }
    // Directories first, then case-insensitive name order.
    entries.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(a.0.to_lowercase().cmp(&b.0.to_lowercase()))
    });

    let mut rows = String::new();
    if url_path != "/" {
        rows.push_str("<tr><td><a href=\"../\">../</a></td><td></td><td></td></tr>\n");
    }
    for (name, is_dir, size, modified) in entries {
        let display = if is_dir {
            format!("{name}/")
        } else {
            name.clone()
        };
        let href = format!("{}{}", encode_uri(&name), if is_dir { "/" } else { "" });
        let size_str = if is_dir {
            String::new()
        } else {
            format_size(size, DECIMAL)
        };
        let date_str = modified
            .and_then(|t| t.format(&Rfc3339).ok())
            .unwrap_or_default();
        rows.push_str(&format!(
            "<tr><td><a href=\"{href}\">{}</a></td><td class=\"size\">{size_str}</td><td class=\"date\">{date_str}</td></tr>\n",
            escape_html(&display)
        ));
    }

    let upload_html = if upload {
        "<form class=\"up\" method=\"post\" action=\"/upload\" enctype=\"multipart/form-data\">\
         <input type=\"file\" name=\"file\" multiple required>\
         <button type=\"submit\">Upload</button></form>"
    } else {
        ""
    };

    let breadcrumb = escape_html(url_path);
    Ok(TEMPLATE
        .replace("{TITLE}", &breadcrumb)
        .replace("{BREADCRUMB}", &breadcrumb)
        .replace("{UPLOAD}", upload_html)
        .replace("{ROWS}", &rows))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Minimal percent-encoding for href segments (space, #, ?, %).
fn encode_uri(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'#' | b'?' | b'%' | b' ' => out.push_str(&format!("%{b:02X}")),
            _ => out.push(b as char),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lists_entries_dirs_first() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("zsub")).unwrap();
        fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let html = render_listing(dir.path(), "/", false).unwrap();
        let dir_pos = html.find("zsub/").unwrap();
        let file_pos = html.find("a.txt").unwrap();
        assert!(dir_pos < file_pos, "directories should sort before files");
        assert!(html.contains("5 B")); // humansize of "hello"
    }

    #[test]
    fn shows_upload_widget_only_when_enabled() {
        let dir = tempdir().unwrap();
        assert!(
            !render_listing(dir.path(), "/", false)
                .unwrap()
                .contains("<form")
        );
        assert!(
            render_listing(dir.path(), "/", true)
                .unwrap()
                .contains("<form")
        );
    }

    #[test]
    fn escapes_html_in_names() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a&b.txt"), b"x").unwrap();
        let html = render_listing(dir.path(), "/", false).unwrap();
        assert!(html.contains("a&amp;b.txt"));
    }
}
