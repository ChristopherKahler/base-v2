use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

pub fn add(
    cwd: &Path,
    ns: &NamespaceConfig,
    name: &str,
    entity_type: &str,
) -> Result<String> {
    let slug = crud::slugify(name);
    let iri = crud::build_iri(ns, "entity", &slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    // Map entity_type string to ontology class
    let rdf_type = match entity_type.to_lowercase().as_str() {
        "person" => format!("{p}:Person"),
        "organization" | "org" => format!("{p}:Organization"),
        _ => format!("{p}:Entity"),
    };

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{iri}> rdf:type {rdf_type} ;\n\
               {p}:name \"{name}\" ;\n\
               {p}:status \"active\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime ;\n\
               {p}:lastActive \"{now}\"^^xsd:dateTime .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(slug)
}

pub fn list(cwd: &Path, ns: &NamespaceConfig) -> Result<()> {
    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?type ?status WHERE {{\n\
           GRAPH ?g {{\n\
             ?e a ?type ;\n\
               {p}:name ?name .\n\
             OPTIONAL {{ ?e {p}:status ?status }}\n\
             FILTER(?type IN ({p}:Entity, {p}:Person, {p}:Organization))\n\
           }}\n\
         }}\n\
         ORDER BY ?name"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rows: Vec<Vec<String>> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                vec![
                    row.get("name").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                    row.get("type").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                    row.get("status").map(|t| crud::term_display(t.into())).unwrap_or_else(|| "-".into()),
                ]
            })
            .collect();

        if rows.is_empty() {
            println!("No entities found.");
            return Ok(());
        }

        println!("| name | type | status |");
        println!("|------|------|--------|");
        for row in &rows {
            println!("| {} | {} | {} |", row[0], row[1], row[2]);
        }
    }
    Ok(())
}

pub fn get(cwd: &Path, ns: &NamespaceConfig, slug: &str) -> Result<()> {
    let iri = crud::build_iri(ns, "entity", slug);
    let sparql = format!(
        "SELECT ?pred ?obj WHERE {{\n\
           GRAPH ?g {{\n\
             <{iri}> ?pred ?obj .\n\
           }}\n\
         }}"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rows: Vec<(String, String)> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                (
                    row.get("pred").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                    row.get("obj").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                )
            })
            .collect();

        if rows.is_empty() {
            eprintln!("Entity '{slug}' not found.");
            return Ok(());
        }

        println!("Entity: {slug}");
        for (pred, obj) in &rows {
            println!("  {pred}: {obj}");
        }
    }
    Ok(())
}

pub fn update(
    cwd: &Path,
    ns: &NamespaceConfig,
    slug: &str,
    status: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    let iri = crud::build_iri(ns, "entity", slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let mut updates = Vec::new();

    if let Some(s) = status {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:status"), &format!("\"{s}\"")));
    }
    if let Some(d) = description {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:description"), &format!("\"{d}\"")));
    }

    updates.push(crud::field_update(&graph, &iri, &format!("{p}:updatedAt"), &format!("\"{now}\"^^xsd:dateTime")));
    updates.push(crud::field_update(&graph, &iri, &format!("{p}:lastActive"), &format!("\"{now}\"^^xsd:dateTime")));

    let sparql = updates.join(" ;\n");
    crud::load_and_mutate(cwd, ns, &sparql)
}
