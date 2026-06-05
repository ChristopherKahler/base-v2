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
    priority: Option<&str>,
    milestone_slug: Option<&str>,
) -> Result<String> {
    let task_slug_part = crud::slugify(name);
    let slug = format!("{project_slug}.{task_slug_part}");
    let task_iri = crud::build_iri(ns, "task", &slug);
    let project_iri = crud::build_iri(ns, "project", project_slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;
    let pri = priority.unwrap_or("medium");

    // Build milestone edge if provided
    let milestone_edge = if let Some(ms) = milestone_slug {
        let ms_iri = crud::build_iri(ns, "milestone", ms);
        format!("<{ms_iri}> {p}:hasTask <{task_iri}> .\n")
    } else {
        String::new()
    };

    let name = crud::escape_sparql_literal(name);

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{task_iri}> rdf:type {p}:Task ;\n\
               {p}:name \"{name}\" ;\n\
               {p}:status \"active\" ;\n\
               {p}:priority \"{pri}\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime ;\n\
               {p}:lastActive \"{now}\"^^xsd:dateTime .\n\
             <{project_iri}> {p}:hasTask <{task_iri}> .\n\
             {milestone_edge}\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(slug)
}

pub fn list(cwd: &Path, ns: &NamespaceConfig, project_slug: Option<&str>, milestone_slug: Option<&str>) -> Result<()> {
    let p = &ns.prefix;
    let sparql = if let Some(ms) = milestone_slug {
        // Filter by milestone
        let ms_iri = crud::build_iri(ns, "milestone", ms);
        format!(
            "SELECT ?name ?status ?priority WHERE {{\n\
               GRAPH ?g {{\n\
                 <{ms_iri}> {p}:hasTask ?task .\n\
                 ?task {p}:name ?name ;\n\
                   {p}:status ?status .\n\
                 OPTIONAL {{ ?task {p}:priority ?priority }}\n\
               }}\n\
             }}\n\
             ORDER BY ?name"
        )
    } else if let Some(ps) = project_slug {
        // Filter by project
        let project_iri = crud::build_iri(ns, "project", ps);
        format!(
            "SELECT ?name ?status ?priority WHERE {{\n\
               GRAPH ?g {{\n\
                 <{project_iri}> {p}:hasTask ?task .\n\
                 ?task {p}:name ?name ;\n\
                   {p}:status ?status .\n\
                 OPTIONAL {{ ?task {p}:priority ?priority }}\n\
               }}\n\
             }}\n\
             ORDER BY ?name"
        )
    } else {
        // All tasks
        format!(
            "SELECT ?name ?status ?priority WHERE {{\n\
               GRAPH ?g {{\n\
                 ?task a {p}:Task ;\n\
                   {p}:name ?name ;\n\
                   {p}:status ?status .\n\
                 OPTIONAL {{ ?task {p}:priority ?priority }}\n\
               }}\n\
             }}\n\
             ORDER BY ?name"
        )
    };

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
            println!("No tasks found.");
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

pub fn done(cwd: &Path, ns: &NamespaceConfig, slug: &str) -> Result<()> {
    let task_iri = crud::build_iri(ns, "task", slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let updates = [
        crud::field_update(&graph, &task_iri, &format!("{p}:status"), "\"completed\""),
        crud::field_update(
            &graph,
            &task_iri,
            &format!("{p}:updatedAt"),
            &format!("\"{now}\"^^xsd:dateTime"),
        ),
        crud::field_update(
            &graph,
            &task_iri,
            &format!("{p}:lastActive"),
            &format!("\"{now}\"^^xsd:dateTime"),
        ),
    ];

    let sparql = updates.join(" ;\n");
    crud::load_and_mutate(cwd, ns, &sparql)
}
