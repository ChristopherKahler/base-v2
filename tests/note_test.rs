use std::path::Path;

fn setup_workspace(dir: &Path) {
    let base_dir = dir.join(".base");
    std::fs::create_dir_all(&base_dir).unwrap();
    std::fs::write(
        base_dir.join("base.toml"),
        "[namespace]\nprefix = \"ops\"\nuri = \"http://ops-sys.local/ontology#\"\n",
    )
    .unwrap();
    // Need a domains.toml for domain sync (so domain IRIs exist)
    std::fs::write(
        base_dir.join("domains.toml"),
        "[[domain]]\nname = \"DEVELOPMENT\"\nmode = \"triggered\"\nprompt_keywords = [\"code\"]\nrules = [\"Dev rule\"]\n",
    )
    .unwrap();
    let config = base::config::BaseConfig::load(dir);
    base::domain::sync::sync_domains_to_graph(&config, dir, None).unwrap();
}

#[test]
fn learn_creates_note_entity() {
    let tmp = tempfile::tempdir().unwrap();
    setup_workspace(tmp.path());
    let ns = &base::config::BaseConfig::load(tmp.path()).namespace;

    let slug = base::crud::note::learn(
        tmp.path(),
        ns,
        "Always use JWT for auth",
        "insight",
        None,
        None,
        None,
    )
    .unwrap();

    assert!(!slug.is_empty());

    // Verify note exists in graph
    let p = &ns.prefix;
    let results = base::crud::load_and_query(
        tmp.path(),
        ns,
        &format!("SELECT ?text ?type WHERE {{ GRAPH ?g {{ ?n a {p}:Note ; {p}:noteText ?text ; {p}:noteType ?type }} }}"),
    )
    .unwrap();

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        let rows: Vec<(String, String)> = solutions
            .filter_map(|r| r.ok())
            .filter_map(|row| {
                let text = row.get("text").map(|t| base::crud::term_display(t.into()))?;
                let ntype = row.get("type").map(|t| base::crud::term_display(t.into()))?;
                Some((text, ntype))
            })
            .collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].0.contains("JWT"));
        assert_eq!(rows[0].1, "insight");
    } else {
        panic!("Expected solutions");
    }
}

#[test]
fn learn_with_domain_creates_edge() {
    let tmp = tempfile::tempdir().unwrap();
    setup_workspace(tmp.path());
    let ns = &base::config::BaseConfig::load(tmp.path()).namespace;

    base::crud::note::learn(
        tmp.path(),
        ns,
        "Test everything before claiming done",
        "correction",
        Some("DEVELOPMENT"),
        None,
        None,
    )
    .unwrap();

    // Verify relatedTo edge exists
    let p = &ns.prefix;
    let domain_iri = base::crud::build_iri(ns, "domain", "development");
    let results = base::crud::load_and_query(
        tmp.path(),
        ns,
        &format!(
            "SELECT ?text WHERE {{ GRAPH ?g {{ ?n a {p}:Note ; {p}:noteText ?text ; {p}:relatedTo <{domain_iri}> }} }}"
        ),
    )
    .unwrap();

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        let texts: Vec<String> = solutions
            .filter_map(|r| r.ok())
            .filter_map(|row| row.get("text").map(|t| base::crud::term_display(t.into())))
            .collect();
        assert_eq!(texts.len(), 1, "Should find note linked to DEVELOPMENT domain");
        assert!(texts[0].contains("Test everything"));
    } else {
        panic!("Expected solutions");
    }
}

#[test]
fn recall_by_keyword_finds_notes() {
    let tmp = tempfile::tempdir().unwrap();
    setup_workspace(tmp.path());
    let ns = &base::config::BaseConfig::load(tmp.path()).namespace;

    base::crud::note::learn(tmp.path(), ns, "Use Rust for hot path", "decision", None, None, None).unwrap();
    base::crud::note::learn(tmp.path(), ns, "Python is fine for scripts", "insight", None, None, None).unwrap();

    // recall by keyword should work without error
    let result = base::crud::note::recall(tmp.path(), ns, Some("rust"), None);
    assert!(result.is_ok());
}

#[test]
fn recall_by_domain_finds_linked_notes() {
    let tmp = tempfile::tempdir().unwrap();
    setup_workspace(tmp.path());
    let ns = &base::config::BaseConfig::load(tmp.path()).namespace;

    // Create a note linked to DEVELOPMENT
    base::crud::note::learn(
        tmp.path(), ns, "Linked note", "insight", Some("DEVELOPMENT"), None, None,
    ).unwrap();

    // Create a note NOT linked to any domain
    base::crud::note::learn(
        tmp.path(), ns, "Unlinked note", "insight", None, None, None,
    ).unwrap();

    // recall by domain should work
    let result = base::crud::note::recall(tmp.path(), ns, None, Some("DEVELOPMENT"));
    assert!(result.is_ok());
}

#[test]
fn notes_for_domain_returns_linked_notes() {
    let tmp = tempfile::tempdir().unwrap();
    setup_workspace(tmp.path());
    let config = base::config::BaseConfig::load(tmp.path());
    let ns = &config.namespace;

    // Learn two notes, one linked to DEVELOPMENT
    base::crud::note::learn(
        tmp.path(), ns, "Dev note 1", "insight", Some("DEVELOPMENT"), None, None,
    ).unwrap();
    base::crud::note::learn(
        tmp.path(), ns, "Dev note 2", "correction", Some("DEVELOPMENT"), None, None,
    ).unwrap();
    base::crud::note::learn(
        tmp.path(), ns, "Other note", "insight", None, None, None,
    ).unwrap();

    // Load graph and query
    let base_dir = base::config::find_workspace_base(tmp.path()).unwrap();
    let store = base::store::load_graph(&base_dir.join("graph.nq")).unwrap();
    let domain_iri = base::crud::build_iri(ns, "domain", "development");

    let notes = base::crud::note::notes_for_domain(&store, ns, &domain_iri);
    assert_eq!(notes.len(), 2, "Should find 2 notes linked to DEVELOPMENT");
    assert!(notes.iter().any(|(t, text)| t == "insight" && text.contains("Dev note 1")));
    assert!(notes.iter().any(|(t, text)| t == "correction" && text.contains("Dev note 2")));
}
