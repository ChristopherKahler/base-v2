use std::fs;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

/// The canonical RDF format for graph persistence.
/// NQuads is immune to the graph-block split corruption bug in oxigraph's
/// TriG serializer (one quad per line, no stateful graph-block machine).
const GRAPH_FORMAT: RdfFormat = RdfFormat::NQuads;

/// Load an NQuads file into a new in-memory store.
/// Each named graph in the file becomes a named graph in the store.
pub fn load_graph(path: &Path) -> Result<Store> {
    let store = Store::new().context("Failed to create in-memory store")?;
    let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    store
        .load_from_reader(GRAPH_FORMAT, reader)
        .with_context(|| format!("Failed to parse graph from {}", path.display()))?;
    Ok(store)
}

/// Load multiple graph files into a single in-memory store (cross-tier query).
pub fn load_graphs(paths: &[&Path]) -> Result<Store> {
    let store = Store::new().context("Failed to create in-memory store")?;
    for path in paths {
        let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = BufReader::new(file);
        store
            .load_from_reader(GRAPH_FORMAT, reader)
            .with_context(|| format!("Failed to parse graph from {}", path.display()))?;
    }
    Ok(store)
}

/// Run a SPARQL query (SELECT or ASK) against the store.
pub fn query(store: &Store, sparql: &str) -> Result<QueryResults> {
    store
        .query(sparql)
        .with_context(|| format!("SPARQL query failed: {sparql}"))
}

/// Serialize the store to NQuads and write atomically (temp + rename).
/// Validates the output by re-parsing before committing the rename.
pub fn write_back(store: &Store, path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .context("Path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory {}", parent.display()))?;

    let tmp_path = path.with_extension("nq.tmp");
    let mut tmp_file = fs::File::create(&tmp_path)
        .with_context(|| format!("Failed to create temp file {}", tmp_path.display()))?;

    store
        .dump_to_writer(RdfSerializer::from_format(GRAPH_FORMAT), &mut tmp_file)
        .context("Failed to serialize store to NQuads")?;

    // Flush and close the file handle before validation.
    drop(tmp_file);

    // Validate: re-parse the written file to catch serializer corruption.
    {
        let check_file = fs::File::open(&tmp_path)
            .with_context(|| format!("Failed to re-open {} for validation", tmp_path.display()))?;
        let check_reader = BufReader::new(check_file);
        let check_store = Store::new().context("Failed to create validation store")?;
        if let Err(e) = check_store.load_from_reader(GRAPH_FORMAT, check_reader) {
            let _ = fs::remove_file(&tmp_path);
            anyhow::bail!(
                "write_back validation failed — serializer produced invalid output, \
                 original file preserved. Parse error: {e}"
            );
        }
    }

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
