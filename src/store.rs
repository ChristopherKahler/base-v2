use std::fs;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

/// Load a TriG file into a new in-memory store.
/// Each named graph in the file becomes a named graph in the store.
pub fn load_graph(path: &Path) -> Result<Store> {
    let store = Store::new().context("Failed to create in-memory store")?;
    let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    store
        .load_from_reader(RdfFormat::TriG, reader)
        .with_context(|| format!("Failed to parse TriG from {}", path.display()))?;
    Ok(store)
}

/// Load multiple TriG files into a single in-memory store (cross-tier query).
pub fn load_graphs(paths: &[&Path]) -> Result<Store> {
    let store = Store::new().context("Failed to create in-memory store")?;
    for path in paths {
        let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = BufReader::new(file);
        store
            .load_from_reader(RdfFormat::TriG, reader)
            .with_context(|| format!("Failed to parse TriG from {}", path.display()))?;
    }
    Ok(store)
}

/// Run a SPARQL query (SELECT or ASK) against the store.
pub fn query(store: &Store, sparql: &str) -> Result<QueryResults> {
    store
        .query(sparql)
        .with_context(|| format!("SPARQL query failed: {sparql}"))
}

/// Serialize the store to TriG and write atomically (temp + rename).
pub fn write_back(store: &Store, path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .context("Path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory {}", parent.display()))?;

    let tmp_path = path.with_extension("trig.tmp");
    let mut tmp_file = fs::File::create(&tmp_path)
        .with_context(|| format!("Failed to create temp file {}", tmp_path.display()))?;

    store
        .dump_to_writer(RdfSerializer::from_format(RdfFormat::TriG), &mut tmp_file)
        .context("Failed to serialize store to TriG")?;

    // Atomic rename
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to rename {} → {}",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}
