use std::path::{Path, PathBuf};

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::{load_queries, BaseConfig};
use crate::ontology;
use crate::store;

pub fn handle(config: &BaseConfig, cwd: &Path) -> Result<()> {
    // Clear session dedup state for fresh session
    if let Some(base_dir) = crate::config::find_workspace_base(cwd) {
        crate::domain::session::SessionState::clear(&base_dir);
    }

    // Try signals first (Phase 5) — primary injection source
    if let Ok(signal_output) = crate::signal::run_signals(cwd, config)
        && !signal_output.is_empty() {
            print!("{signal_output}");
            return Ok(());
        }

    // Fallback: ad-hoc queries from queries.toml (Phase 1 behavior)
    let trig_files = discover_trig_files(cwd);

    if trig_files.is_empty() {
        return Ok(());
    }

    let paths: Vec<&Path> = trig_files.iter().map(|p| p.as_path()).collect();
    let graph = store::load_graphs(&paths)?;

    ontology::load_vocabulary(&graph, &config.namespace)?;

    let queries = load_queries(cwd, config);
    let mut output = String::new();

    for qdef in &queries {
        let sparql = format!(
            "PREFIX {p}: <{u}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
             PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n\
             {body}",
            p = config.namespace.prefix,
            u = config.namespace.uri,
            body = qdef.sparql,
        );

        if let Ok(results) = store::query(&graph, &sparql) {
            let section = format_results(results, &qdef.format, &qdef.description);
            if !section.is_empty() {
                output.push_str(&section);
                output.push('\n');
            }
        }
    }

    if !output.is_empty() {
        print!("{}", output.trim_end());
    }

    Ok(())
}

/// Discover TriG files from global and workspace tiers.
fn discover_trig_files(cwd: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Global tier: ~/.base-gbl/graph.trig
    if let Some(home) = dirs::home_dir() {
        let global = home.join(".base-gbl").join("graph.trig");
        if global.exists() {
            files.push(global);
        }
    }

    // Workspace tier: walk upward from cwd to find .base/graph.trig
    let mut dir = cwd.to_path_buf();
    loop {
        let ws = dir.join(".base").join("graph.trig");
        if ws.exists() {
            files.push(ws);
            break;
        }
        if !dir.pop() {
            break;
        }
    }

    files
}

/// Format SPARQL SELECT results according to the query's format type.
fn format_results(results: QueryResults, format: &str, description: &str) -> String {
    let QueryResults::Solutions(solutions) = results else {
        return String::new();
    };

    let vars: Vec<String> = solutions
        .variables()
        .iter()
        .map(|v| v.as_str().to_string())
        .collect();

    let rows: Vec<Vec<String>> = solutions
        .filter_map(|r| r.ok())
        .map(|row| {
            vars.iter()
                .map(|v| {
                    row.get(v.as_str())
                        .map(|term| term_display(term.into()))
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    if rows.is_empty() {
        return String::new();
    }

    let mut out = format!("[{description}]\n");

    match format {
        "table" => {
            out.push_str(&format!("| {} |\n", vars.join(" | ")));
            out.push_str(&format!(
                "|{}|\n",
                vars.iter().map(|_| "---").collect::<Vec<_>>().join("|")
            ));
            for row in &rows {
                out.push_str(&format!("| {} |\n", row.join(" | ")));
            }
        }
        "prose" => {
            let vals: Vec<String> = rows.iter().map(|r| r.join(" ")).collect();
            out.push_str(&vals.join(". "));
            out.push('\n');
        }
        _ => {
            // Default: list
            for row in &rows {
                out.push_str(&format!("- {}\n", row.join(" — ")));
            }
        }
    }

    out
}

/// Extract a human-readable string from an RDF term.
fn term_display(term: oxigraph::model::TermRef<'_>) -> String {
    use oxigraph::model::TermRef;
    match term {
        TermRef::Literal(l) => l.value().to_string(),
        TermRef::NamedNode(n) => {
            let iri = n.as_str();
            // Extract local name after # or last /
            iri.rfind('#')
                .or_else(|| iri.rfind('/'))
                .map(|pos| iri[pos + 1..].to_string())
                .unwrap_or_else(|| iri.to_string())
        }
        TermRef::BlankNode(b) => format!("_:{}", b.as_str()),
        #[allow(unreachable_patterns)]
        _ => term.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_nothing_in_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let files = discover_trig_files(tmp.path());
        assert!(files.is_empty());
    }

    #[test]
    fn discover_finds_workspace_trig() {
        let tmp = tempfile::tempdir().unwrap();
        let base_dir = tmp.path().join(".base");
        std::fs::create_dir_all(&base_dir).unwrap();
        std::fs::write(base_dir.join("graph.trig"), "# empty").unwrap();

        let files = discover_trig_files(tmp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with(".base/graph.trig"));
    }
}
