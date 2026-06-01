use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;

use crate::config::BaseConfig;
use crate::store;

pub fn handle(config: &BaseConfig, cwd: &Path, event: &serde_json::Value) -> Result<()> {
    let file_paths = extract_file_paths(event);
    if file_paths.is_empty() {
        return Ok(());
    }

    let trig_path = find_workspace_trig(cwd);
    let Some(trig_path) = trig_path else {
        return Ok(()); // No workspace graph yet — nothing to update
    };

    let graph = store::load_graph(&trig_path)?;
    let now = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let mut updated = false;

    for file_path in &file_paths {
        let sparql = format!(
            "PREFIX {p}: <{u}>\n\
             PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n\
             DELETE {{ GRAPH ?g {{ ?entity {p}:lastActive ?old }} }}\n\
             INSERT {{ GRAPH ?g {{ ?entity {p}:lastActive \"{now}\"^^xsd:dateTime }} }}\n\
             WHERE {{\n\
               GRAPH ?g {{\n\
                 ?entity {p}:path ?path .\n\
                 FILTER(STRSTARTS(\"{file}\", STR(?path)))\n\
                 OPTIONAL {{ ?entity {p}:lastActive ?old }}\n\
               }}\n\
             }}",
            p = config.namespace.prefix,
            u = config.namespace.uri,
            file = file_path.display(),
        );

        if graph.update(&sparql).is_ok() {
            updated = true;
        }
    }

    if updated {
        store::write_back(&graph, &trig_path)?;
    }

    // post-tool-use never emits to stdout — it only mutates the graph
    Ok(())
}

/// Extract file paths from the hook event JSON.
/// Handles common Claude Code tool shapes: Edit, Write, Read.
fn extract_file_paths(event: &serde_json::Value) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // tool_input.file_path — used by Edit, Write, Read
    if let Some(fp) = event
        .get("tool_input")
        .and_then(|ti| ti.get("file_path"))
        .and_then(|v| v.as_str())
    {
        paths.push(PathBuf::from(fp));
    }

    // tool_input.path — alternative field name
    if let Some(fp) = event
        .get("tool_input")
        .and_then(|ti| ti.get("path"))
        .and_then(|v| v.as_str())
    {
        paths.push(PathBuf::from(fp));
    }

    // tool_input.command — Bash commands may reference files, but we skip those
    // (too noisy, too hard to parse reliably)

    paths
}

/// Find the workspace .base/graph.trig by walking upward from cwd.
fn find_workspace_trig(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd.to_path_buf();
    loop {
        let candidate = dir.join(".base").join("graph.trig");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_file_path_from_edit_event() {
        let event = serde_json::json!({
            "tool_name": "Edit",
            "tool_input": {
                "file_path": "/home/user/project/src/main.rs",
                "old_string": "foo",
                "new_string": "bar"
            }
        });
        let paths = extract_file_paths(&event);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].to_str().unwrap(), "/home/user/project/src/main.rs");
    }

    #[test]
    fn extract_returns_empty_for_unknown_tool() {
        let event = serde_json::json!({
            "tool_name": "WebSearch",
            "tool_input": {
                "query": "rust oxigraph"
            }
        });
        let paths = extract_file_paths(&event);
        assert!(paths.is_empty());
    }

    #[test]
    fn find_workspace_trig_walks_up() {
        let tmp = tempfile::tempdir().unwrap();
        let base_dir = tmp.path().join(".base");
        std::fs::create_dir_all(&base_dir).unwrap();
        std::fs::write(base_dir.join("graph.trig"), "# test").unwrap();

        let sub = tmp.path().join("deep").join("nested");
        std::fs::create_dir_all(&sub).unwrap();

        let found = find_workspace_trig(&sub);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with(".base/graph.trig"));
    }

    #[test]
    fn find_workspace_trig_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let found = find_workspace_trig(tmp.path());
        assert!(found.is_none());
    }
}
