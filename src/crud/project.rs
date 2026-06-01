use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

pub fn add(
    cwd: &Path,
    ns: &NamespaceConfig,
    name: &str,
    status: &str,
    path: Option<&str>,
) -> Result<String> {
    let slug = crud::slugify(name);
    let iri = crud::build_iri(ns, "project", &slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let ws_iri = crud::build_iri(ns, "workspace", &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;
    let project_path = path
        .map(|s| s.to_string())
        .unwrap_or_else(|| cwd.to_string_lossy().to_string());

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{iri}> rdf:type {p}:Project ;\n\
               {p}:name \"{name}\" ;\n\
               {p}:status \"{status}\" ;\n\
               {p}:path \"{project_path}\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime ;\n\
               {p}:lastActive \"{now}\"^^xsd:dateTime ;\n\
               {p}:belongsTo <{ws_iri}> .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;

    // Auto-create domain trigger with path matching (filesystem-first, no keywords by default)
    auto_create_domain(cwd, name, &project_path)?;

    // Link project to its domain in the graph
    let domain_slug = crud::slugify(name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);
    let link_sparql = format!(
        "INSERT DATA {{ GRAPH <{graph}> {{ <{iri}> {p}:hasDomain <{domain_iri}> }} }}"
    );
    let _ = crud::load_and_mutate(cwd, ns, &link_sparql);

    Ok(slug)
}

/// Auto-create a domain trigger entry in the nearest domains.toml.
/// Default: path-based matching. No keywords unless user adds them later.
fn auto_create_domain(cwd: &Path, project_name: &str, project_path: &str) -> Result<()> {
    // Add a path trigger via the existing add_trigger mechanism
    crate::domain::add_trigger(cwd, project_name, None, Some(project_path))?;
    Ok(())
}

pub fn list(cwd: &Path, ns: &NamespaceConfig) -> Result<()> {
    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?status ?priority ?lastActive WHERE {{\n\
           GRAPH ?g {{\n\
             ?proj a {p}:Project ;\n\
               {p}:name ?name ;\n\
               {p}:status ?status .\n\
             OPTIONAL {{ ?proj {p}:priority ?priority }}\n\
             OPTIONAL {{ ?proj {p}:lastActive ?lastActive }}\n\
           }}\n\
         }}\n\
         ORDER BY ?name"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let vars: Vec<String> = solutions
            .variables()
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        let rows: Vec<Vec<String>> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                vars.iter()
                    .map(|v| {
                        row.get(v.as_str())
                            .map(|t| crud::term_display(t.into()))
                            .unwrap_or_else(|| "-".into())
                    })
                    .collect()
            })
            .collect();

        if rows.is_empty() {
            println!("No projects found.");
            return Ok(());
        }

        println!("| {} |", vars.join(" | "));
        println!("|{}|", vars.iter().map(|_| "---").collect::<Vec<_>>().join("|"));
        for row in &rows {
            println!("| {} |", row.join(" | "));
        }
    }
    Ok(())
}

pub fn get(cwd: &Path, ns: &NamespaceConfig, slug: &str) -> Result<()> {
    let iri = crud::build_iri(ns, "project", slug);
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
            eprintln!("Project '{slug}' not found.");
            return Ok(());
        }

        println!("Project: {slug}");
        for (pred, obj) in &rows {
            // Skip rdf:type display name — show it as "type"
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
    blocked_by: Option<&str>,
    next_action: Option<&str>,
) -> Result<()> {
    let iri = crud::build_iri(ns, "project", slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let mut updates = Vec::new();

    if let Some(s) = status {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:status"), &format!("\"{s}\"")));
    }
    if let Some(b) = blocked_by {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:blockedBy"), &format!("\"{b}\"")));
    }
    if let Some(n) = next_action {
        updates.push(crud::field_update(&graph, &iri, &format!("{p}:nextAction"), &format!("\"{n}\"")));
    }

    // Always update timestamps
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
