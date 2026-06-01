use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

/// Create a note (memory entry) with optional relational edges.
pub fn learn(
    cwd: &Path,
    ns: &NamespaceConfig,
    text: &str,
    note_type: &str,
    domain: Option<&str>,
    project: Option<&str>,
    entity: Option<&str>,
) -> Result<String> {
    let slug = crud::slugify(text);
    let iri = crud::build_iri(ns, "note", &slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

    // Build relatedTo edges
    let mut edge_triples = String::new();
    if let Some(d) = domain {
        let domain_iri = crud::build_iri(ns, "domain", &crud::slugify(d));
        edge_triples.push_str(&format!("      <{iri}> {p}:relatedTo <{domain_iri}> .\n"));
    }
    if let Some(proj) = project {
        let proj_iri = crud::build_iri(ns, "project", &crud::slugify(proj));
        edge_triples.push_str(&format!("      <{iri}> {p}:relatedTo <{proj_iri}> .\n"));
    }
    if let Some(ent) = entity {
        let ent_iri = crud::build_iri(ns, "entity", &crud::slugify(ent));
        edge_triples.push_str(&format!("      <{iri}> {p}:relatedTo <{ent_iri}> .\n"));
    }

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{iri}> rdf:type {p}:Note ;\n\
               {p}:noteText \"{escaped_text}\" ;\n\
               {p}:noteType \"{note_type}\" ;\n\
               {p}:status \"active\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime .\n\
         {edge_triples}\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(slug)
}

/// Search notes by keyword text match and/or domain linkage.
pub fn recall(
    cwd: &Path,
    ns: &NamespaceConfig,
    keyword: Option<&str>,
    domain: Option<&str>,
) -> Result<()> {
    let p = &ns.prefix;

    let sparql = match (keyword, domain) {
        (Some(kw), Some(dom)) => {
            let kw_lower = kw.to_lowercase();
            let domain_iri = crud::build_iri(ns, "domain", &crud::slugify(dom));
            format!(
                "SELECT ?text ?type ?created WHERE {{\n\
                   GRAPH ?g {{\n\
                     {{ ?n a {p}:Note ; {p}:noteText ?text ; {p}:noteType ?type .\n\
                        OPTIONAL {{ ?n {p}:createdAt ?created }}\n\
                        FILTER(CONTAINS(LCASE(STR(?text)), \"{kw_lower}\"))\n\
                     }} UNION {{\n\
                        ?n a {p}:Note ; {p}:noteText ?text ; {p}:noteType ?type ; {p}:relatedTo <{domain_iri}> .\n\
                        OPTIONAL {{ ?n {p}:createdAt ?created }}\n\
                     }}\n\
                     ?n {p}:status \"active\" .\n\
                   }}\n\
                 }}"
            )
        }
        (Some(kw), None) => {
            let kw_lower = kw.to_lowercase();
            format!(
                "SELECT ?text ?type ?created WHERE {{\n\
                   GRAPH ?g {{\n\
                     ?n a {p}:Note ; {p}:noteText ?text ; {p}:noteType ?type ; {p}:status \"active\" .\n\
                     OPTIONAL {{ ?n {p}:createdAt ?created }}\n\
                     FILTER(CONTAINS(LCASE(STR(?text)), \"{kw_lower}\"))\n\
                   }}\n\
                 }}"
            )
        }
        (None, Some(dom)) => {
            let domain_iri = crud::build_iri(ns, "domain", &crud::slugify(dom));
            format!(
                "SELECT ?text ?type ?created WHERE {{\n\
                   GRAPH ?g {{\n\
                     ?n a {p}:Note ; {p}:noteText ?text ; {p}:noteType ?type ; {p}:status \"active\" ;\n\
                       {p}:relatedTo <{domain_iri}> .\n\
                     OPTIONAL {{ ?n {p}:createdAt ?created }}\n\
                   }}\n\
                 }}"
            )
        }
        (None, None) => {
            eprintln!("Provide --keyword and/or --domain");
            return Ok(());
        }
    };

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rows: Vec<Vec<String>> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                vec![
                    row.get("type")
                        .map(|t| crud::term_display(t.into()))
                        .unwrap_or_default(),
                    row.get("text")
                        .map(|t| crud::term_display(t.into()))
                        .unwrap_or_default(),
                    row.get("created")
                        .map(|t| crud::term_display(t.into()))
                        .unwrap_or_else(|| "-".into()),
                ]
            })
            .collect();

        if rows.is_empty() {
            println!("No notes found.");
            return Ok(());
        }

        println!("| type | text | created |");
        println!("|------|------|---------|");
        for row in &rows {
            println!("| {} | {} | {} |", row[0], row[1], row[2]);
        }
    }
    Ok(())
}

/// Query notes linked to a specific domain IRI. Returns (type, text) pairs.
/// Used by hook injection to surface notes alongside domain rules.
pub fn notes_for_domain(
    store: &oxigraph::store::Store,
    ns: &NamespaceConfig,
    domain_iri: &str,
) -> Vec<(String, String)> {
    let p = &ns.prefix;
    let pfx = crud::prefixes(ns);

    let sparql = format!(
        "{pfx}\n\
         SELECT ?text ?type WHERE {{\n\
           GRAPH ?g {{\n\
             ?n a {p}:Note ;\n\
               {p}:noteText ?text ;\n\
               {p}:noteType ?type ;\n\
               {p}:status \"active\" ;\n\
               {p}:relatedTo <{domain_iri}> .\n\
           }}\n\
         }}"
    );

    match crate::store::query(store, &sparql) {
        Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) => solutions
            .filter_map(|r| r.ok())
            .filter_map(|row| {
                let text = row.get("text").map(|t| crud::term_display(t.into()))?;
                let note_type = row.get("type").map(|t| crud::term_display(t.into()))?;
                if text.is_empty() {
                    None
                } else {
                    Some((note_type, text))
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}
