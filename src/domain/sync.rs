use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::{BaseConfig, NamespaceConfig};
use crate::crud;
use crate::domain;

// ─── Sync stats ─────────────────────────────────────────────

pub struct SyncStats {
    pub domains: usize,
    pub rules: usize,
    pub decisions: usize,
}

// ─── Domain → graph sync ────────────────────────────────────

/// Sync domains.toml domains/rules into the graph as ops:Domain and ops:Rule entities.
/// Optionally migrates decisions from a carl.json file.
/// Idempotent: deletes existing domain/rule triples before re-inserting.
pub fn sync_domains_to_graph(
    config: &BaseConfig,
    cwd: &Path,
    carl_json_path: Option<&Path>,
) -> Result<SyncStats> {
    let ns = &config.namespace;
    let (store, trig_path) = crud::load_workspace_store(cwd)?;
    let ws_slug = crud::workspace_slug(cwd);
    let graph = crud::workspace_graph_iri(ns, &ws_slug);
    let pfx = crud::prefixes(ns);
    let p = &ns.prefix;

    let domains = domain::load_domains(cwd);
    let mut total_rules = 0usize;

    for domain_def in &domains {
        let domain_slug = crud::slugify(&domain_def.name);
        let domain_iri = crud::build_iri(ns, "domain", &domain_slug);

        // Delete existing triples for this domain and its rules
        let delete_sparql = format!(
            "{pfx}\n\
             DELETE {{\n\
               GRAPH <{graph}> {{\n\
                 <{domain_iri}> ?dp ?do .\n\
                 ?rule ?rp ?ro .\n\
               }}\n\
             }}\n\
             WHERE {{\n\
               GRAPH <{graph}> {{\n\
                 <{domain_iri}> ?dp ?do .\n\
                 OPTIONAL {{\n\
                   <{domain_iri}> {p}:hasRule ?rule .\n\
                   ?rule ?rp ?ro .\n\
                 }}\n\
               }}\n\
             }}"
        );
        // Ignore errors on delete (domain may not exist yet)
        let _ = store.update(&delete_sparql);

        // Insert domain entity
        let prompt_kw_triples: String = domain_def
            .prompt_keywords
            .iter()
            .map(|kw| format!("      {p}:promptKeyword \"{}\" ;\n", escape_sparql(kw)))
            .collect();

        let file_kw_triples: String = domain_def
            .file_keywords
            .iter()
            .map(|kw| format!("      {p}:fileKeyword \"{}\" ;\n", escape_sparql(kw)))
            .collect();

        let path_triples: String = domain_def
            .paths
            .iter()
            .map(|path| format!("      {p}:triggerPath \"{}\" ;\n", escape_sparql(path)))
            .collect();

        let now = crud::now_iso();
        let domain_insert = format!(
            "{pfx}\n\
             INSERT DATA {{\n\
               GRAPH <{graph}> {{\n\
                 <{domain_iri}> rdf:type {p}:Domain ;\n\
                   {p}:name \"{}\" ;\n\
                   {p}:status \"{}\" ;\n\
             {prompt_kw_triples}\
             {file_kw_triples}\
             {path_triples}\
                   {p}:updatedAt \"{now}\"^^xsd:dateTime .\n\
               }}\n\
             }}",
            escape_sparql(&domain_def.name),
            escape_sparql(&domain_def.mode),
        );
        store
            .update(&domain_insert)
            .with_context(|| format!("Failed to insert domain '{}'", domain_def.name))?;

        // Insert rules
        for (i, rule_text) in domain_def.rules.iter().enumerate() {
            let rule_iri = crud::build_iri(ns, "rule", &format!("{domain_slug}/{i}"));
            let rule_insert = format!(
                "{pfx}\n\
                 INSERT DATA {{\n\
                   GRAPH <{graph}> {{\n\
                     <{rule_iri}> rdf:type {p}:Rule ;\n\
                       {p}:ruleText \"{}\" ;\n\
                       {p}:priority \"{i}\" .\n\
                     <{domain_iri}> {p}:hasRule <{rule_iri}> .\n\
                   }}\n\
                 }}",
                escape_sparql(rule_text),
            );
            store
                .update(&rule_insert)
                .with_context(|| format!("Failed to insert rule {i} for domain '{}'", domain_def.name))?;
            total_rules += 1;
        }
    }

    // Migrate decisions from carl.json if provided
    let total_decisions = if let Some(carl_path) = carl_json_path {
        sync_carl_decisions(&store, ns, &graph, &pfx, carl_path)?
    } else {
        0
    };

    crate::store::write_back(&store, &trig_path)?;

    Ok(SyncStats {
        domains: domains.len(),
        rules: total_rules,
        decisions: total_decisions,
    })
}

// ─── Carl.json decision migration ───────────────────────────

/// Minimal carl.json structure — only what we need for decision migration.
#[derive(Deserialize)]
struct CarlJson {
    #[serde(default)]
    domains: Vec<CarlDomain>,
}

#[derive(Deserialize)]
struct CarlDomain {
    name: String,
    #[serde(default)]
    decisions: Vec<CarlDecision>,
}

#[derive(Deserialize)]
struct CarlDecision {
    decision: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    recall: String,
}

fn sync_carl_decisions(
    store: &oxigraph::store::Store,
    ns: &NamespaceConfig,
    graph: &str,
    pfx: &str,
    carl_path: &Path,
) -> Result<usize> {
    let content = std::fs::read_to_string(carl_path)
        .with_context(|| format!("Failed to read carl.json at {}", carl_path.display()))?;
    let carl: CarlJson = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse carl.json at {}", carl_path.display()))?;

    let p = &ns.prefix;
    let now = crud::now_iso();
    let mut count = 0usize;

    for domain in &carl.domains {
        let domain_slug = crud::slugify(&domain.name);
        let domain_iri = crud::build_iri(ns, "domain", &domain_slug);

        for decision in &domain.decisions {
            let dec_slug = format!(
                "{}.{}",
                domain_slug,
                crud::slugify(&decision.decision)
            );
            let dec_iri = crud::build_iri(ns, "decision", &dec_slug);

            // Delete existing (idempotent)
            let delete = format!(
                "{pfx}\n\
                 DELETE {{ GRAPH <{graph}> {{ <{dec_iri}> ?p ?o }} }}\n\
                 WHERE {{ GRAPH <{graph}> {{ <{dec_iri}> ?p ?o }} }}"
            );
            let _ = store.update(&delete);

            let recall_triple = if decision.recall.is_empty() {
                String::new()
            } else {
                format!("      {p}:recall \"{}\" ;\n", escape_sparql(&decision.recall))
            };

            let insert = format!(
                "{pfx}\n\
                 INSERT DATA {{\n\
                   GRAPH <{graph}> {{\n\
                     <{dec_iri}> rdf:type {p}:Decision ;\n\
                       {p}:name \"{}\" ;\n\
                       {p}:rationale \"{}\" ;\n\
                 {recall_triple}\
                       {p}:status \"active\" ;\n\
                       {p}:createdAt \"{now}\"^^xsd:dateTime .\n\
                     <{domain_iri}> {p}:hasDecision <{dec_iri}> .\n\
                   }}\n\
                 }}",
                escape_sparql(&decision.decision),
                escape_sparql(&decision.rationale),
            );
            store
                .update(&insert)
                .with_context(|| format!("Failed to insert decision for domain '{}'", domain.name))?;
            count += 1;
        }
    }

    Ok(count)
}

// ─── Helpers ────────────────────────────────────────────────

/// Escape special characters for SPARQL string literals.
fn escape_sparql(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BaseConfig;

    fn setup_workspace(dir: &Path) {
        let base_dir = dir.join(".base");
        std::fs::create_dir_all(&base_dir).unwrap();
        std::fs::write(
            base_dir.join("domains.toml"),
            r#"
[[domain]]
name = "GLOBAL"
mode = "always"
prompt_keywords = []
rules = ["Rule 0: always be helpful", "Rule 1: never lie"]

[[domain]]
name = "DEVELOPMENT"
mode = "triggered"
prompt_keywords = ["write code", "fix bug"]
file_keywords = ["use crate", "impl"]
paths = ["src/"]
rules = ["Dev rule: test everything"]
"#,
        )
        .unwrap();
    }

    #[test]
    fn sync_creates_domain_and_rule_entities() {
        let tmp = tempfile::tempdir().unwrap();
        setup_workspace(tmp.path());
        let config = BaseConfig::load(tmp.path());

        let stats = sync_domains_to_graph(&config, tmp.path(), None).unwrap();
        assert_eq!(stats.domains, 2);
        assert_eq!(stats.rules, 3); // 2 GLOBAL + 1 DEVELOPMENT

        // Verify domains exist in graph
        let ns = &config.namespace;
        let p = &ns.prefix;
        let results = crud::load_and_query(
            tmp.path(),
            ns,
            &format!("SELECT ?name WHERE {{ GRAPH ?g {{ ?d a {p}:Domain ; {p}:name ?name }} }}"),
        )
        .unwrap();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            let names: Vec<String> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| row.get("name").map(|t| crud::term_display(t.into())))
                .collect();
            assert!(names.contains(&"GLOBAL".to_string()));
            assert!(names.contains(&"DEVELOPMENT".to_string()));
        } else {
            panic!("Expected solutions");
        }
    }

    #[test]
    fn sync_creates_rules_with_edges() {
        let tmp = tempfile::tempdir().unwrap();
        setup_workspace(tmp.path());
        let config = BaseConfig::load(tmp.path());

        sync_domains_to_graph(&config, tmp.path(), None).unwrap();

        let ns = &config.namespace;
        let p = &ns.prefix;
        let domain_iri = crud::build_iri(ns, "domain", "global");

        let results = crud::load_and_query(
            tmp.path(),
            ns,
            &format!(
                "SELECT ?text WHERE {{ GRAPH ?g {{ <{domain_iri}> {p}:hasRule ?r . ?r {p}:ruleText ?text }} }}"
            ),
        )
        .unwrap();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            let texts: Vec<String> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| row.get("text").map(|t| crud::term_display(t.into())))
                .collect();
            assert_eq!(texts.len(), 2);
            assert!(texts.iter().any(|t| t.contains("always be helpful")));
            assert!(texts.iter().any(|t| t.contains("never lie")));
        } else {
            panic!("Expected solutions");
        }
    }

    #[test]
    fn sync_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        setup_workspace(tmp.path());
        let config = BaseConfig::load(tmp.path());

        // Sync twice
        sync_domains_to_graph(&config, tmp.path(), None).unwrap();
        let stats = sync_domains_to_graph(&config, tmp.path(), None).unwrap();
        assert_eq!(stats.domains, 2);
        assert_eq!(stats.rules, 3);

        // Count total rules — should be 3, not 6
        let ns = &config.namespace;
        let p = &ns.prefix;
        let results = crud::load_and_query(
            tmp.path(),
            ns,
            &format!("SELECT (COUNT(?r) AS ?cnt) WHERE {{ GRAPH ?g {{ ?r a {p}:Rule }} }}"),
        )
        .unwrap();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            let row = solutions.filter_map(|r| r.ok()).next().unwrap();
            let cnt = crud::term_display(row.get("cnt").unwrap().into());
            assert_eq!(cnt, "3", "Should have exactly 3 rules, not duplicates");
        }
    }

    #[test]
    fn sync_carl_decisions_creates_entities() {
        let tmp = tempfile::tempdir().unwrap();
        setup_workspace(tmp.path());

        // Write a minimal carl.json
        let carl_path = tmp.path().join("carl.json");
        std::fs::write(
            &carl_path,
            r#"{
  "domains": [
    {
      "name": "GLOBAL",
      "decisions": [
        {
          "decision": "Use JWT for auth",
          "rationale": "Stateless, scalable",
          "recall": "When discussing auth"
        }
      ]
    }
  ]
}"#,
        )
        .unwrap();

        let config = BaseConfig::load(tmp.path());
        let stats = sync_domains_to_graph(&config, tmp.path(), Some(&carl_path)).unwrap();
        assert_eq!(stats.decisions, 1);

        // Verify decision linked to domain
        let ns = &config.namespace;
        let p = &ns.prefix;
        let domain_iri = crud::build_iri(ns, "domain", "global");

        let results = crud::load_and_query(
            tmp.path(),
            ns,
            &format!(
                "SELECT ?name WHERE {{ GRAPH ?g {{ <{domain_iri}> {p}:hasDecision ?d . ?d {p}:name ?name }} }}"
            ),
        )
        .unwrap();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            let names: Vec<String> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| row.get("name").map(|t| crud::term_display(t.into())))
                .collect();
            assert_eq!(names.len(), 1);
            assert!(names[0].contains("JWT"));
        } else {
            panic!("Expected solutions");
        }
    }

    #[test]
    fn backward_compat_legacy_keywords_field() {
        let tmp = tempfile::tempdir().unwrap();
        let base_dir = tmp.path().join(".base");
        std::fs::create_dir_all(&base_dir).unwrap();
        // Legacy format: `keywords` instead of `prompt_keywords`
        std::fs::write(
            base_dir.join("domains.toml"),
            r#"
[[domain]]
name = "LEGACY"
mode = "triggered"
keywords = ["old keyword"]
rules = ["Legacy rule"]
"#,
        )
        .unwrap();

        let config = BaseConfig::load(tmp.path());
        let stats = sync_domains_to_graph(&config, tmp.path(), None).unwrap();
        assert_eq!(stats.domains, 1);
        assert_eq!(stats.rules, 1);

        // Verify prompt keyword was synced from legacy field
        let ns = &config.namespace;
        let p = &ns.prefix;
        let domain_iri = crud::build_iri(ns, "domain", "legacy");
        let results = crud::load_and_query(
            tmp.path(),
            ns,
            &format!(
                "SELECT ?kw WHERE {{ GRAPH ?g {{ <{domain_iri}> {p}:promptKeyword ?kw }} }}"
            ),
        )
        .unwrap();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            let kws: Vec<String> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| row.get("kw").map(|t| crud::term_display(t.into())))
                .collect();
            assert_eq!(kws.len(), 1);
            assert_eq!(kws[0], "old keyword");
        }
    }
}
