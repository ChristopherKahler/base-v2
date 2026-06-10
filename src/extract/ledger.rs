use crate::config::NamespaceConfig;
use crate::crud;
use std::path::Path;

/// Ledger entry from .paul/ledger.toml
#[derive(Debug, serde::Deserialize)]
struct LedgerEntry {
    action: String,
    phase: Option<u32>,
    plan: Option<String>,
    at: String,
    note: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct LedgerFile {
    entry: Option<Vec<LedgerEntry>>,
}

/// Extract ledger.toml entries into graph triples.
/// Called directly from sync pipeline (not via the file-walk route).
/// Returns number of entries extracted.
pub fn extract_ledger(
    cwd: &Path,
    store: &oxigraph::store::Store,
    ns: &NamespaceConfig,
    graph_iri: &str,
) -> usize {
    let ledger_path = cwd.join(".paul/ledger.toml");
    if !ledger_path.exists() {
        return 0;
    }

    let content = match std::fs::read_to_string(&ledger_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let ledger: LedgerFile = match toml::from_str(&content) {
        Ok(l) => l,
        Err(_) => return 0,
    };

    let entries = match ledger.entry {
        Some(e) => e,
        None => return 0,
    };

    let p = &ns.prefix;
    let prefixes = crud::prefixes(ns);
    let mut count = 0;

    for entry in &entries {
        // Build deterministic IRI from content hash
        let hash_input = format!(
            "{}:{}:{}:{}",
            entry.action,
            entry.phase.unwrap_or(0),
            entry.plan.as_deref().unwrap_or(""),
            entry.at,
        );
        let hash = simple_hash(&hash_input);
        let iri = format!("{}ledger/{}", ns.uri, hash);

        // Delete existing triples for this IRI (idempotent)
        let del = format!(
            "{prefixes}\nDELETE WHERE {{ GRAPH <{graph_iri}> {{ <{iri}> ?p ?o }} }}"
        );
        let _ = store.update(&del);

        // Build triples
        let mut body = String::new();
        body.push_str(&format!("    <{iri}> rdf:type {p}:LedgerEntry .\n"));
        body.push_str(&format!(
            "    <{iri}> {p}:action \"{}\" .\n",
            crud::escape_sparql_literal(&entry.action)
        ));
        body.push_str(&format!(
            "    <{iri}> {p}:timestamp \"{}\" .\n",
            entry.at
        ));

        if let Some(phase) = entry.phase {
            body.push_str(&format!("    <{iri}> {p}:phase \"{}\" .\n", crud::escape_sparql_literal(&phase.to_string())));
        }
        if let Some(ref plan) = entry.plan {
            body.push_str(&format!("    <{iri}> {p}:plan \"{}\" .\n", crud::escape_sparql_literal(plan)));
        }
        if let Some(ref note) = entry.note
            && !note.is_empty() {
                body.push_str(&format!(
                    "    <{iri}> {p}:note \"{}\" .\n",
                    crud::escape_sparql_literal(note)
                ));
            }

        // Link to project — read name from paul.toml, fall back to workspace slug
        let project_name = read_paul_toml_name(cwd)
            .unwrap_or_else(|| crud::workspace_slug(cwd));
        let project_slug = crud::slugify(&project_name);
        let project_iri = crud::build_iri(ns, "project", &project_slug);
        body.push_str(&format!(
            "    <{iri}> {p}:belongsTo <{project_iri}> .\n"
        ));

        let now = crud::now_iso();
        body.push_str(&format!(
            "    <{iri}> {p}:lastExtracted \"{now}\"^^xsd:dateTime .\n"
        ));

        let ins = format!(
            "{prefixes}\nINSERT DATA {{ GRAPH <{graph_iri}> {{\n{body}}} }}"
        );
        if store.update(&ins).is_ok() {
            count += 1;
        }
    }

    count
}

/// Read project name from .paul/paul.toml
fn read_paul_toml_name(cwd: &Path) -> Option<String> {
    let toml_path = cwd.join(".paul/paul.toml");
    let content = std::fs::read_to_string(toml_path).ok()?;
    let table: toml::Table = toml::from_str(&content).ok()?;
    table.get("name")?.as_str().map(String::from)
}

/// Simple deterministic hash for IRI generation.
fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{hash:016x}")
}
