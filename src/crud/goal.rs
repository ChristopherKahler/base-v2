use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

pub fn add(cwd: &Path, ns: &NamespaceConfig, name: &str, target: &str) -> Result<String> {
    let slug = crud::slugify(name);
    let iri = crud::build_iri(ns, "goal", &slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{iri}> rdf:type {p}:Goal ;\n\
               {p}:name \"{name}\" ;\n\
               {p}:description \"{target}\" ;\n\
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
        "SELECT ?name ?description ?status WHERE {{\n\
           GRAPH ?g {{\n\
             ?goal a {p}:Goal ;\n\
               {p}:name ?name .\n\
             OPTIONAL {{ ?goal {p}:description ?description }}\n\
             OPTIONAL {{ ?goal {p}:status ?status }}\n\
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
                    row.get("description").map(|t| crud::term_display(t.into())).unwrap_or_else(|| "-".into()),
                    row.get("status").map(|t| crud::term_display(t.into())).unwrap_or_else(|| "-".into()),
                ]
            })
            .collect();

        if rows.is_empty() {
            println!("No goals found.");
            return Ok(());
        }

        println!("| name | target | status |");
        println!("|------|--------|--------|");
        for row in &rows {
            println!("| {} | {} | {} |", row[0], row[1], row[2]);
        }
    }
    Ok(())
}

pub fn update(
    cwd: &Path,
    ns: &NamespaceConfig,
    slug: &str,
    status: Option<&str>,
    target: Option<&str>,
) -> Result<()> {
    let iri = crud::build_iri(ns, "goal", slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let mut updates = Vec::new();

    if let Some(s) = status {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:status"), &format!("\"{s}\"")));
    }
    if let Some(t) = target {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:description"), &format!("\"{t}\"")));
    }

    updates.push(crud::field_update(&graph, &iri, &format!("{p}:updatedAt"), &format!("\"{now}\"^^xsd:dateTime")));
    updates.push(crud::field_update(&graph, &iri, &format!("{p}:lastActive"), &format!("\"{now}\"^^xsd:dateTime")));

    let sparql = updates.join(" ;\n");
    crud::load_and_mutate(cwd, ns, &sparql)
}
