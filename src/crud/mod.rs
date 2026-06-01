pub mod decision;
pub mod entity;
pub mod goal;
pub mod note;
pub mod project;
pub mod rule;
pub mod reminder;
pub mod task;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

use crate::config::NamespaceConfig;

// ─── IRI building ────────────────────────────────────────────

/// Build an entity IRI: `{ns.uri}{entity_type}/{slug}`.
pub fn build_iri(ns: &NamespaceConfig, entity_type: &str, slug: &str) -> String {
    format!("{}{}/{}", ns.uri, entity_type, slug)
}

/// Build the workspace graph IRI: `{ns.uri}graph/ws/{workspace_slug}`.
pub fn workspace_graph_iri(ns: &NamespaceConfig, workspace_slug: &str) -> String {
    format!("{}graph/ws/{}", ns.uri, workspace_slug)
}

/// Derive workspace slug from the workspace root directory name.
pub fn workspace_slug(cwd: &Path) -> String {
    if let Some(base_dir) = crate::config::find_workspace_base(cwd) {
        base_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(slugify)
            .unwrap_or_else(|| "default".into())
    } else {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .map(slugify)
            .unwrap_or_else(|| "default".into())
    }
}

/// Convert a name to a URL-safe slug: lowercase, non-alphanumeric→hyphens, deduped.
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ─── SPARQL helpers ──────────────────────────────────────────

/// Standard PREFIX block for SPARQL operations.
pub fn prefixes(ns: &NamespaceConfig) -> String {
    format!(
        "PREFIX {p}: <{u}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>",
        p = ns.prefix,
        u = ns.uri
    )
}

/// Build a DELETE+INSERT operation for a single field.
/// Handles the case where the field doesn't exist yet (OPTIONAL).
pub fn field_update(
    graph_iri: &str,
    subject_iri: &str,
    pred: &str,
    new_value: &str,
) -> String {
    format!(
        "DELETE {{ GRAPH <{g}> {{ <{s}> {pred} ?old }} }}\n\
         INSERT {{ GRAPH <{g}> {{ <{s}> {pred} {val} }} }}\n\
         WHERE {{\n\
           GRAPH <{g}> {{ <{s}> a ?type }}\n\
           OPTIONAL {{ GRAPH <{g}> {{ <{s}> {pred} ?old }} }}\n\
         }}",
        g = graph_iri,
        s = subject_iri,
        val = new_value,
    )
}

/// Current timestamp as ISO 8601 with timezone.
pub fn now_iso() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

// ─── Graph pipeline ──────────────────────────────────────────

/// Find or create the workspace `.base/` directory.
fn find_or_create_base(cwd: &Path) -> Result<PathBuf> {
    if let Some(base) = crate::config::find_workspace_base(cwd) {
        Ok(base)
    } else {
        let base = cwd.join(".base");
        std::fs::create_dir_all(&base).context("creating .base/ directory")?;
        Ok(base)
    }
}

/// Load the workspace store. Creates .base/ and an empty store if graph.trig doesn't exist.
pub fn load_workspace_store(cwd: &Path) -> Result<(Store, PathBuf)> {
    let base_dir = find_or_create_base(cwd)?;
    let trig_path = base_dir.join("graph.trig");

    let store = if trig_path.exists() {
        crate::store::load_graph(&trig_path)?
    } else {
        Store::new().context("creating empty store")?
    };

    Ok((store, trig_path))
}

/// Load store, execute SPARQL UPDATE, write back atomically.
pub fn load_and_mutate(cwd: &Path, ns: &NamespaceConfig, sparql: &str) -> Result<()> {
    let (store, trig_path) = load_workspace_store(cwd)?;
    let full_sparql = format!("{}\n{}", prefixes(ns), sparql);
    store
        .update(&full_sparql)
        .with_context(|| format!("SPARQL update failed: {full_sparql}"))?;
    crate::store::write_back(&store, &trig_path)
}

/// Load workspace graph and run a SPARQL SELECT query.
pub fn load_and_query(cwd: &Path, ns: &NamespaceConfig, sparql: &str) -> Result<QueryResults> {
    let base_dir = crate::config::find_workspace_base(cwd)
        .context("no .base/ directory found")?;
    let trig_path = base_dir.join("graph.trig");
    let store = crate::store::load_graph(&trig_path)?;
    let full_sparql = format!("{}\n{}", prefixes(ns), sparql);
    crate::store::query(&store, &full_sparql)
}

// ─── Display helpers ─────────────────────────────────────────

/// Extract a human-readable string from an RDF term.
pub fn term_display(term: oxigraph::model::TermRef<'_>) -> String {
    use oxigraph::model::TermRef;
    match term {
        TermRef::Literal(l) => l.value().to_string(),
        TermRef::NamedNode(n) => {
            let iri = n.as_str();
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
    fn slugify_works() {
        assert_eq!(slugify("My Cool Project"), "my-cool-project");
        assert_eq!(slugify("CaseGate v2"), "casegate-v2");
        assert_eq!(slugify("hello--world"), "hello-world");
        assert_eq!(slugify("  spaced  "), "spaced");
    }

    #[test]
    fn build_iri_follows_scheme() {
        let ns = NamespaceConfig::default();
        assert_eq!(
            build_iri(&ns, "project", "casegate-v2"),
            "http://ops-sys.local/ontology#project/casegate-v2"
        );
    }

    #[test]
    fn workspace_graph_iri_correct() {
        let ns = NamespaceConfig::default();
        assert_eq!(
            workspace_graph_iri(&ns, "chris-ai-systems"),
            "http://ops-sys.local/ontology#graph/ws/chris-ai-systems"
        );
    }
}
