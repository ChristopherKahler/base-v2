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

/// Auto-migrate a legacy graph.trig to graph.nq if present.
/// Loads the TriG file, writes it as NQuads, and removes the old file.
/// Returns Ok(true) if migration happened, Ok(false) if no legacy file found.
pub fn migrate_trig_to_nq(nq_path: &Path) -> Result<bool> {
    let trig_path = nq_path.with_extension("trig");
    // If the caller passed a .trig path directly, trig_path == nq_path and the
    // "remove old file" branch below would delete the file it was asked to load.
    if trig_path == nq_path {
        return Ok(false);
    }
    if !trig_path.exists() {
        return Ok(false);
    }

    // If graph.nq already exists alongside graph.trig, just remove the old one
    if nq_path.exists() {
        let _ = fs::remove_file(&trig_path);
        return Ok(false);
    }

    eprintln!("Migrating {} → {} ...", trig_path.display(), nq_path.display());

    // Load from TriG format
    let store = Store::new().context("Failed to create migration store")?;
    let file = fs::File::open(&trig_path)
        .with_context(|| format!("Failed to open legacy {}", trig_path.display()))?;
    let reader = BufReader::new(file);
    store
        .load_from_reader(RdfFormat::TriG, reader)
        .with_context(|| format!(
            "Failed to parse legacy {}. File may be corrupted — \
             delete it and run `base sync` to rebuild.",
            trig_path.display()
        ))?;

    // Write as NQuads using write_back (includes validation)
    write_back(&store, nq_path)?;

    // Remove old TriG file
    let _ = fs::remove_file(&trig_path);

    eprintln!("Migration complete. Legacy graph.trig removed.");
    Ok(true)
}

/// Load an NQuads file into a new in-memory store.
/// Auto-migrates legacy graph.trig if graph.nq doesn't exist yet.
pub fn load_graph(path: &Path) -> Result<Store> {
    // Auto-migrate legacy TriG if needed
    migrate_trig_to_nq(path)?;

    let store = Store::new().context("Failed to create in-memory store")?;
    let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    store
        .load_from_reader(GRAPH_FORMAT, reader)
        .with_context(|| format!("Failed to parse graph from {}", path.display()))?;
    Ok(store)
}

/// Load multiple graph files into a single in-memory store (cross-tier query).
/// Auto-migrates legacy TriG files if needed.
pub fn load_graphs(paths: &[&Path]) -> Result<Store> {
    let store = Store::new().context("Failed to create in-memory store")?;
    for path in paths {
        // Auto-migrate legacy TriG if needed
        migrate_trig_to_nq(path)?;

        let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = BufReader::new(file);
        store
            .load_from_reader(GRAPH_FORMAT, reader)
            .with_context(|| format!("Failed to parse graph from {}", path.display()))?;
    }
    Ok(store)
}

/// Load a merged graph from global (~/.base-gbl/.base/graph.nq) and workspace tiers
/// into one store so SPARQL queries span all tiers. Returns None only if neither
/// graph exists (fail-open). Call ONCE per hook invocation and share the store.
pub fn load_merged(cwd: &Path) -> Option<Store> {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let global_nq = home.join(".base-gbl").join(".base").join("graph.nq");
        if global_nq.exists() {
            paths.push(global_nq);
        }
    }

    if let Some(base_dir) = crate::config::find_workspace_base(cwd) {
        let ws_nq = base_dir.join("graph.nq");
        if ws_nq.exists() {
            paths.push(ws_nq);
        }
    }

    if paths.is_empty() {
        return None;
    }

    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    load_graphs(&path_refs).ok()
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
