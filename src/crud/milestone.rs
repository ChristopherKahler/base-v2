use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

pub fn add(
    cwd: &Path,
    ns: &NamespaceConfig,
    project_slug: &str,
    name: &str,
    description: Option<&str>,
) -> Result<String> {
    let ms_slug_part = crud::slugify(name);
    let slug = format!("{project_slug}.{ms_slug_part}");
    let ms_iri = crud::build_iri(ns, "milestone", &slug);
    let project_iri = crud::build_iri(ns, "project", project_slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;
    let name = crud::escape_sparql_literal(name);
    let desc = crud::escape_sparql_literal(description.unwrap_or(""));

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{ms_iri}> rdf:type {p}:Milestone ;\n\
               {p}:name \"{name}\" ;\n\
               {p}:status \"active\" ;\n\
               {p}:description \"{desc}\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime ;\n\
               {p}:lastActive \"{now}\"^^xsd:dateTime ;\n\
               {p}:belongsTo <{project_iri}> .\n\
             <{project_iri}> {p}:hasMilestone <{ms_iri}> .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(slug)
}

pub fn list(cwd: &Path, ns: &NamespaceConfig, project_slug: Option<&str>) -> Result<()> {
    let p = &ns.prefix;
    let sparql = if let Some(ps) = project_slug {
        let project_iri = crud::build_iri(ns, "project", ps);
        format!(
            "SELECT ?ms ?name ?status ?description WHERE {{\n\
               GRAPH ?g {{\n\
                 <{project_iri}> {p}:hasMilestone ?ms .\n\
                 ?ms {p}:name ?name ;\n\
                   {p}:status ?status .\n\
                 OPTIONAL {{ ?ms {p}:description ?description }}\n\
               }}\n\
             }}\n\
             ORDER BY ?name"
        )
    } else {
        format!(
            "SELECT ?ms ?name ?status ?description WHERE {{\n\
               GRAPH ?g {{\n\
                 ?ms a {p}:Milestone ;\n\
                   {p}:name ?name ;\n\
                   {p}:status ?status .\n\
                 OPTIONAL {{ ?ms {p}:description ?description }}\n\
               }}\n\
             }}\n\
             ORDER BY ?name"
        )
    };

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rows: Vec<(String, String, String, String)> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                let slug = row
                    .get("ms")
                    .map(|t| {
                        let full = crud::term_display(t.into());
                        // Strip "milestone/" prefix from IRI fragment
                        full.strip_prefix("milestone/")
                            .unwrap_or(&full)
                            .to_string()
                    })
                    .unwrap_or_default();
                let name = row
                    .get("name")
                    .map(|t| crud::term_display(t.into()))
                    .unwrap_or_default();
                let status = row
                    .get("status")
                    .map(|t| crud::term_display(t.into()))
                    .unwrap_or_default();
                let desc = row
                    .get("description")
                    .map(|t| crud::term_display(t.into()))
                    .unwrap_or_else(|| "-".into());
                (slug, name, status, desc)
            })
            .collect();

        if rows.is_empty() {
            println!("No milestones found.");
            return Ok(());
        }

        println!("| slug | name | status | description |");
        println!("|---|---|---|---|");
        for (slug, name, status, desc) in &rows {
            println!("| {slug} | {name} | {status} | {desc} |");
        }
    }
    Ok(())
}

pub fn get(cwd: &Path, ns: &NamespaceConfig, slug: &str) -> Result<()> {
    let iri = crud::build_iri(ns, "milestone", slug);
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
                let pred = row
                    .get("pred")
                    .map(|t| crud::term_display(t.into()))
                    .unwrap_or_default();
                let obj = row
                    .get("obj")
                    .map(|t| crud::term_display(t.into()))
                    .unwrap_or_default();
                (pred, obj)
            })
            .collect();

        if rows.is_empty() {
            eprintln!("Milestone '{slug}' not found.");
            return Ok(());
        }

        println!("Milestone: {slug}");
        for (pred, obj) in &rows {
            let label = if pred == "type" {
                "type".to_string()
            } else {
                pred.clone()
            };
            println!("  {label}: {obj}");
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
    let iri = crud::build_iri(ns, "milestone", slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let mut updates = Vec::new();

    if let Some(s) = status {
        updates.push(crud::field_update(
            &graph,
            &iri,
            &format!("{p}:status"),
            &format!("\"{s}\""),
        ));
    }
    if let Some(d) = description {
        updates.push(crud::field_update(
            &graph,
            &iri,
            &format!("{p}:description"),
            &format!("\"{d}\""),
        ));
    }

    updates.push(crud::field_update(
        &graph,
        &iri,
        &format!("{p}:updatedAt"),
        &format!("\"{now}\"^^xsd:dateTime"),
    ));
    updates.push(crud::field_update(
        &graph,
        &iri,
        &format!("{p}:lastActive"),
        &format!("\"{now}\"^^xsd:dateTime"),
    ));

    let sparql = updates.join(" ;\n");
    crud::load_and_mutate(cwd, ns, &sparql)
}
