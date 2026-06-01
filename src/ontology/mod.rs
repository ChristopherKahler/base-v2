use anyhow::Result;
use oxigraph::store::Store;
use oxigraph::io::RdfFormat;

/// The ops: vocabulary, embedded at compile time.
pub const OPS_TTL: &str = include_str!("ops.ttl");

/// Load the ops: vocabulary into the store's default graph.
pub fn load_vocabulary(store: &Store) -> Result<()> {
    store.load_from_reader(RdfFormat::Turtle, OPS_TTL.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_embeds_and_parses() {
        let store = Store::new().unwrap();
        load_vocabulary(&store).unwrap();
        // Verify at least some triples loaded (classes + predicates)
        assert!(store.len().unwrap() > 0, "Vocabulary should load triples");
    }
}
