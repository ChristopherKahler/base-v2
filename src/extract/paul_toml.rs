use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::NamespaceConfig;
use crate::crud;

// ─── paul.toml schema ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaulToml {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    #[serde(default)]
    pub phase: Option<Phase>,
    #[serde(default)]
    pub r#loop: Option<Loop>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub satellite: Option<Satellite>,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Deserialize)]
pub struct Milestone {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct Phase {
    #[serde(default)]
    pub number: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct Loop {
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default = "default_loop_position")]
    pub position: String,
}

fn default_loop_position() -> String {
    "IDLE".into()
}

#[derive(Debug, Deserialize)]
pub struct Satellite {
    #[serde(default = "default_groom")]
    pub groom: bool,
}

fn default_groom() -> bool {
    true
}

// ─── Scanner ────────────────────────────────────────────────

/// Scan workspace for .paul/paul.toml files. Returns (path_to_toml, parsed_toml) pairs.
pub fn scan_paul_projects(workspace_root: &Path) -> Vec<(PathBuf, PaulToml)> {
    let mut results = Vec::new();

    // Scan apps/ and clients/ subdirectories
    for subdir in &["apps", "clients"] {
        let dir = workspace_root.join(subdir);
        if dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&dir)
        {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    results.extend(try_load_paul_toml(&entry.path()));
                }
            }
        }
    }

    // Check workspace root itself
    results.extend(try_load_paul_toml(workspace_root));

    results
}

fn try_load_paul_toml(project_dir: &Path) -> Option<(PathBuf, PaulToml)> {
    let toml_path = project_dir.join(".paul").join("paul.toml");
    if !toml_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&toml_path).ok()?;
    let parsed: PaulToml = toml::from_str(&content).ok()?;
    Some((toml_path, parsed))
}

// ─── Graph ingestion ────────────────────────────────────────

pub struct IngestStats {
    pub scanned: usize,
    pub registered: usize,
}

/// Ingest all scanned paul.toml projects into the graph. Idempotent: delete + re-insert.
pub fn ingest_paul_projects(
    cwd: &Path,
    ns: &NamespaceConfig,
    projects: &[(PathBuf, PaulToml)],
) -> Result<IngestStats> {
    let (store, trig_path) = crud::load_workspace_store(cwd)?;
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let pfx = crud::prefixes(ns);
    let p = &ns.prefix;
    let now = crud::now_iso();

    let mut registered = 0usize;

    for (_toml_path, paul) in projects {
        let slug = crud::slugify(&paul.name);
        let iri = crud::build_iri(ns, "project", &slug);

        // Delete existing triples for this project (idempotent)
        let delete = format!(
            "{pfx}\n\
             DELETE {{ GRAPH <{graph}> {{ <{iri}> ?p ?o }} }}\n\
             WHERE {{ GRAPH <{graph}> {{ <{iri}> ?p ?o }} }}"
        );
        let _ = store.update(&delete);

        // Build milestone/phase/loop description
        let mut extra_triples = String::new();

        if !paul.path.is_empty() {
            extra_triples.push_str(&format!(
                "      <{iri}> {p}:path \"{}\" .\n",
                escape(&paul.path)
            ));
        }

        if let Some(ref ms) = paul.milestone {
            extra_triples.push_str(&format!(
                "      <{iri}> {p}:description \"Milestone: {} ({}) [{}]\" .\n",
                escape(&ms.name),
                escape(&ms.version),
                escape(&ms.status)
            ));
        }

        if let Some(ref phase) = paul.phase {
            extra_triples.push_str(&format!(
                "      <{iri}> {p}:nextAction \"Phase {}: {} [{}]\" .\n",
                phase.number,
                escape(&phase.name),
                escape(&phase.status)
            ));
        }

        // Tag edges → domain association
        for tag in &paul.tags {
            let domain_iri = crud::build_iri(ns, "domain", &crud::slugify(tag));
            extra_triples.push_str(&format!(
                "      <{iri}> {p}:hasDomain <{domain_iri}> .\n"
            ));
        }

        let insert = format!(
            "{pfx}\n\
             INSERT DATA {{\n\
               GRAPH <{graph}> {{\n\
                 <{iri}> rdf:type {p}:Project ;\n\
                   {p}:name \"{}\" ;\n\
                   {p}:status \"{}\" ;\n\
                   {p}:lastActive \"{now}\"^^xsd:dateTime ;\n\
                   {p}:updatedAt \"{now}\"^^xsd:dateTime .\n\
             {extra_triples}\
               }}\n\
             }}",
            escape(&paul.name),
            escape(&paul.status),
        );

        store.update(&insert)
            .with_context(|| format!("Failed to ingest paul project '{}'", paul.name))?;
        registered += 1;
    }

    crate::store::write_back(&store, &trig_path)?;

    Ok(IngestStats {
        scanned: projects.len(),
        registered,
    })
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
