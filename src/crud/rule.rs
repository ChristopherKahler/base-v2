use std::path::Path;

use anyhow::{Context, Result};
use oxigraph::sparql::QueryResults;

use crate::config::NamespaceConfig;
use crate::crud;

/// Add a rule to a domain in the graph.
pub fn add(
    cwd: &Path,
    ns: &NamespaceConfig,
    domain_name: &str,
    rule_text: &str,
) -> Result<u32> {
    let p = &ns.prefix;
    let domain_slug = crud::slugify(domain_name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);

    // Find next rule index for this domain
    let next_index = next_rule_index(cwd, ns, &domain_iri)?;
    let rule_iri = crud::build_iri(ns, "rule", &format!("{domain_slug}/{next_index}"));
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);

    let escaped = rule_text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

    // Ensure domain exists
    let ensure_domain = format!(
        "INSERT {{\n\
           GRAPH <{graph}> {{\n\
             <{domain_iri}> rdf:type {p}:Domain ; {p}:name \"{domain_name}\" .\n\
           }}\n\
         }}\n\
         WHERE {{\n\
           FILTER NOT EXISTS {{ GRAPH <{graph}> {{ <{domain_iri}> a {p}:Domain }} }}\n\
         }}"
    );
    let _ = crud::load_and_mutate(cwd, ns, &ensure_domain);

    // Insert rule with edge to domain
    let sparql = format!(
        "INSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{rule_iri}> rdf:type {p}:Rule ;\n\
               {p}:ruleText \"{escaped}\" ;\n\
               {p}:priority \"{next_index}\" .\n\
             <{domain_iri}> {p}:hasRule <{rule_iri}> .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(next_index)
}

/// List rules for a domain from the graph.
pub fn list(cwd: &Path, ns: &NamespaceConfig, domain_name: &str) -> Result<()> {
    let p = &ns.prefix;
    let domain_slug = crud::slugify(domain_name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);

    let sparql = format!(
        "SELECT ?text ?pri WHERE {{\n\
           GRAPH ?g {{\n\
             <{domain_iri}> {p}:hasRule ?rule .\n\
             ?rule {p}:ruleText ?text .\n\
             OPTIONAL {{ ?rule {p}:priority ?pri }}\n\
           }}\n\
         }}\n\
         ORDER BY ?pri"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results {
        let rules: Vec<(String, String)> = solutions
            .filter_map(|r| r.ok())
            .filter_map(|row| {
                let text = row.get("text").map(|t| crud::term_display(t.into()))?;
                let pri = row.get("pri").map(|t| crud::term_display(t.into())).unwrap_or_default();
                Some((pri, text))
            })
            .collect();

        if rules.is_empty() {
            println!("No rules for domain '{domain_name}'.");
            return Ok(());
        }

        println!("[{domain_name}] {} rules:", rules.len());
        for (pri, text) in &rules {
            println!("  {pri}. {text}");
        }
    }
    Ok(())
}

/// Remove a rule by index from a domain.
pub fn remove(cwd: &Path, ns: &NamespaceConfig, domain_name: &str, index: u32) -> Result<()> {
    let p = &ns.prefix;
    let domain_slug = crud::slugify(domain_name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);
    let rule_iri = crud::build_iri(ns, "rule", &format!("{domain_slug}/{index}"));
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);

    let sparql = format!(
        "DELETE {{\n\
           GRAPH <{graph}> {{\n\
             <{rule_iri}> ?p ?o .\n\
             <{domain_iri}> {p}:hasRule <{rule_iri}> .\n\
           }}\n\
         }}\n\
         WHERE {{\n\
           GRAPH <{graph}> {{\n\
             <{rule_iri}> ?p ?o .\n\
           }}\n\
         }}"
    );

    crud::load_and_mutate(cwd, ns, &sparql)?;
    Ok(())
}

/// Find the next available rule index for a domain.
fn next_rule_index(cwd: &Path, ns: &NamespaceConfig, domain_iri: &str) -> Result<u32> {
    let p = &ns.prefix;
    let sparql = format!(
        "SELECT (COUNT(?rule) AS ?cnt) WHERE {{\n\
           GRAPH ?g {{\n\
             <{domain_iri}> {p}:hasRule ?rule .\n\
           }}\n\
         }}"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    if let QueryResults::Solutions(solutions) = results
        && let Some(Ok(row)) = solutions.into_iter().next()
        && let Some(term) = row.get("cnt")
    {
        let cnt_str = crud::term_display(term.into());
        return cnt_str.parse::<u32>().context("parsing rule count");
    }
    Ok(0)
}
