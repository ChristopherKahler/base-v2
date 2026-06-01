use oxigraph::sparql::QueryResults;

use base::config::NamespaceConfig;
use base::crud;

fn ns() -> NamespaceConfig { NamespaceConfig::default() }

#[test]
fn log_decision_creates_triples() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = crud::decision::log(
        tmp.path(), &ns(), "dev", "Use JWT", "Stateless auth", Some("auth, tokens"),
    ).unwrap();
    assert_eq!(slug, "dev.use-jwt");

    let trig_path = tmp.path().join(".base").join("graph.trig");
    let store = base::store::load_graph(&trig_path).unwrap();
    let sparql = format!(
        "PREFIX {p}: <{u}>\nASK {{ GRAPH ?g {{ ?d a {p}:Decision ; {p}:name \"Use JWT\" ; {p}:rationale \"Stateless auth\" }} }}",
        p = ns().prefix, u = ns().uri,
    );
    match store.query(&sparql).unwrap() {
        QueryResults::Boolean(yes) => assert!(yes, "Decision should exist"),
        _ => panic!("Expected boolean"),
    }
}

#[test]
fn search_finds_matching_decision() {
    let tmp = tempfile::tempdir().unwrap();
    crud::decision::log(tmp.path(), &ns(), "auth", "Use JWT tokens", "Fast auth", None).unwrap();
    crud::decision::log(tmp.path(), &ns(), "db", "Use Postgres", "Reliable", None).unwrap();

    // Search should not error — results go to stdout
    let result = crud::decision::search(tmp.path(), &ns(), "JWT");
    assert!(result.is_ok());
}
