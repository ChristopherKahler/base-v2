use crate::config::NamespaceConfig;

/// Extract YAML frontmatter from a markdown file into (predicate, value) triples.
/// Returns None if no valid frontmatter found.
pub fn extract(content: &str, file_path: &str, ns: &NamespaceConfig) -> Option<Vec<(String, String)>> {
    let fm = parse_frontmatter(content)?;
    if fm.is_empty() {
        return None;
    }

    let p = &ns.prefix;
    let mut triples = Vec::new();

    // Always: type and path
    triples.push(("rdf:type".into(), format!("{p}:Document")));
    triples.push((format!("{p}:path"), format!("\"{file_path}\"")));

    // Map frontmatter fields to predicates
    for (key, value) in &fm {
        match key.to_lowercase().as_str() {
            "name" | "title" => {
                triples.push((format!("{p}:name"), format!("\"{}\"", escape(value))));
            }
            "description" | "about" | "summary" => {
                triples.push((format!("{p}:description"), format!("\"{}\"", escape(value))));
            }
            "status" => {
                triples.push((format!("{p}:status"), format!("\"{}\"", escape(value))));
            }
            "priority" => {
                triples.push((format!("{p}:priority"), format!("\"{}\"", escape(value))));
            }
            "tags" => {
                // Tags as comma-separated in description
                triples.push((format!("{p}:description"), format!("\"tags: {}\"", escape(value))));
            }
            _ => {
                // Other fields as description with key prefix
                triples.push((
                    format!("{p}:description"),
                    format!("\"{}: {}\"", key, escape(value)),
                ));
            }
        }
    }

    // If no name was set from frontmatter, derive from filename
    if !fm.iter().any(|(k, _)| k == "name" || k == "title") {
        let filename = file_path
            .rsplit('/')
            .next()
            .unwrap_or(file_path)
            .trim_end_matches(".md");
        triples.push((format!("{p}:name"), format!("\"{}\"", filename)));
    }

    Some(triples)
}

/// Parse YAML frontmatter between --- delimiters.
/// Returns key-value pairs. Handles simple `key: value` lines only.
fn parse_frontmatter(content: &str) -> Option<Vec<(String, String)>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;
    let fm_text = &after_first[..end_pos];

    let mut pairs = Vec::new();
    for line in fm_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() && !value.is_empty() {
                pairs.push((key, value));
            }
        }
    }

    if pairs.is_empty() {
        None
    } else {
        Some(pairs)
    }
}

/// Escape special characters for SPARQL string literals.
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_frontmatter() {
        let content = "---\ntitle: My Doc\nstatus: active\n---\n\n# Content";
        let ns = NamespaceConfig::default();
        let triples = extract(content, "docs/test.md", &ns).unwrap();
        assert!(triples.iter().any(|(p, v)| p.contains("name") && v.contains("My Doc")));
        assert!(triples.iter().any(|(p, v)| p.contains("status") && v.contains("active")));
        assert!(triples.iter().any(|(p, v)| p.contains("path") && v.contains("docs/test.md")));
    }

    #[test]
    fn no_frontmatter_returns_none() {
        let content = "# Just a heading\n\nSome text";
        let ns = NamespaceConfig::default();
        assert!(extract(content, "test.md", &ns).is_none());
    }

    #[test]
    fn empty_frontmatter_returns_none() {
        let content = "---\n---\n\nContent";
        let ns = NamespaceConfig::default();
        assert!(extract(content, "test.md", &ns).is_none());
    }
}
