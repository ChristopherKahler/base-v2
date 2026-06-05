use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

pub fn log(
    cwd: &Path,
    ns: &NamespaceConfig,
    domain: &str,
    decision_text: &str,
    rationale: &str,
    recall: Option<&str>,
) -> Result<String> {
    let slug = format!("{}.{}", crud::slugify(domain), crud::slugify(decision_text));
    let iri = crud::build_iri(ns, "decision", &slug);
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let now = crud::now_iso();
    let p = &ns.prefix;

    let decision_text = crud::escape_sparql_literal(decision_text);
    let rationale = crud::escape_sparql_literal(rationale);

    let recall_triple = recall
        .map(|r| {
            let r = crud::escape_sparql_literal(r);
            format!("      {p}:recall \"{r}\" ;\n")
        })
        .unwrap_or_default();

    let domain_slug = crud::slugify(domain);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);

    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{iri}> rdf:type {p}:Decision ;\n\
               {p}:name \"{decision_text}\" ;\n\
               {p}:rationale \"{rationale}\" ;\n\
         {recall_triple}\
               {p}:status \"active\" ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime ;\n\
               {p}:lastActive \"{now}\"^^xsd:dateTime .\n\
             <{domain_iri}> {p}:hasDecision <{iri}> .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(slug)
}

pub fn search(cwd: &Path, ns: &NamespaceConfig, keyword: &str) -> Result<()> {
    let p = &ns.prefix;
    let kw_lower = keyword.to_lowercase();
    let sparql = format!(
        "SELECT ?name ?rationale ?recall WHERE {{\n\
           GRAPH ?g {{\n\
             ?d a {p}:Decision ;\n\
               {p}:name ?name ;\n\
               {p}:rationale ?rationale .\n\
             OPTIONAL {{ ?d {p}:recall ?recall }}\n\
             FILTER(\n\
               CONTAINS(LCASE(STR(?name)), \"{kw_lower}\") ||\n\
               CONTAINS(LCASE(STR(?rationale)), \"{kw_lower}\") ||\n\
               CONTAINS(LCASE(STR(?recall)), \"{kw_lower}\")\n\
             )\n\
           }}\n\
         }}"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rows: Vec<Vec<String>> = solutions
            .filter_map(|r| r.ok())
            .map(|row| {
                vec![
                    row.get("name").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                    row.get("rationale").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                    row.get("recall").map(|t| crud::term_display(t.into())).unwrap_or_else(|| "-".into()),
                ]
            })
            .collect();

        if rows.is_empty() {
            println!("No decisions matching '{keyword}'.");
            return Ok(());
        }

        println!("| decision | rationale | recall |");
        println!("|----------|-----------|--------|");
        for row in &rows {
            println!("| {} | {} | {} |", row[0], row[1], row[2]);
        }
    }
    Ok(())
}
