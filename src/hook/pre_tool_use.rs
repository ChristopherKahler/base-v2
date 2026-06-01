use std::path::{Path, PathBuf};

use anyhow::Result;
use oxigraph::model::TermRef;

use crate::config::BaseConfig;
use crate::crud;
use crate::domain;
use crate::domain::session::SessionState;

/// PreToolUse: see file path in tool call → match file_keywords + path triggers → inject rules BEFORE tool executes.
pub fn handle(config: &BaseConfig, cwd: &Path, event: &serde_json::Value) -> Result<()> {
    let file_paths = extract_file_paths(event);
    if file_paths.is_empty() {
        return Ok(());
    }

    let domains = domain::load_domains(cwd);
    if domains.is_empty() {
        return Ok(());
    }

    let base_dir = crate::config::find_workspace_base(cwd);
    let session = base_dir
        .as_deref()
        .map(SessionState::load)
        .unwrap_or_default();

    // Match domains using file paths (path triggers) and file_keywords (content scan)
    let file_path_strings: Vec<String> = file_paths
        .iter()
        .filter_map(|p| p.to_str().map(String::from))
        .collect();

    let matched = match_by_file(&domains, &file_path_strings, &session);
    if matched.is_empty() {
        return Ok(());
    }

    // Ensure graph is synced
    crate::hook::user_prompt_submit::ensure_domain_sync_pub(config, cwd);

    let graph_store = load_workspace_graph(cwd);

    let mut output = String::new();
    for domain_def in &matched {
        let rules_text = match &graph_store {
            Some(store) => query_rules_from_graph(store, config, domain_def),
            None => format_toml_rules(domain_def),
        };

        if rules_text.is_empty() {
            continue;
        }

        // No dedup on pre-tool-use — rules should be present for each tool call
        // (the tool call is the trigger, not the prompt)
        output.push_str(&rules_text);
        output.push('\n');
    }

    if !output.is_empty() {
        print!("{}", output.trim_end());
    }

    Ok(())
}

/// Match domains by file path triggers and file_keywords against file content.
fn match_by_file<'a>(
    domains: &'a [domain::DomainDef],
    file_paths: &[String],
    session: &SessionState,
) -> Vec<&'a domain::DomainDef> {
    let _ = session; // reserved for future file-level dedup

    domains
        .iter()
        .filter(|d| {
            // Skip always-on (those fire on user-prompt-submit, not here)
            if d.is_always() {
                return false;
            }

            // Path match: any file path starts with or contains a domain path trigger
            let path_hit = d.paths.iter().any(|dp| {
                file_paths
                    .iter()
                    .any(|fp| fp.starts_with(dp) || fp.contains(dp))
            });

            // File keyword match: check if any file_keywords appear in the file paths
            // (lightweight — full content scan would require reading the file)
            let file_kw_hit = d.file_keywords.iter().any(|kw| {
                file_paths
                    .iter()
                    .any(|fp| fp.to_lowercase().contains(&kw.to_lowercase()))
            });

            path_hit || file_kw_hit
        })
        .collect()
}

/// Query rules for a domain from the graph. Returns formatted text.
fn query_rules_from_graph(
    store: &oxigraph::store::Store,
    config: &BaseConfig,
    domain_def: &domain::DomainDef,
) -> String {
    let ns = &config.namespace;
    let p = &ns.prefix;
    let domain_slug = crud::slugify(&domain_def.name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);
    let pfx = crud::prefixes(ns);

    let sparql = format!(
        "{pfx}\n\
         SELECT ?text WHERE {{\n\
           GRAPH ?g {{\n\
             <{domain_iri}> {p}:hasRule ?rule .\n\
             ?rule {p}:ruleText ?text .\n\
             OPTIONAL {{ ?rule {p}:priority ?pri }}\n\
           }}\n\
         }}\n\
         ORDER BY ?pri"
    );

    match crate::store::query(store, &sparql) {
        Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) => {
            let rules: Vec<String> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| {
                    row.get("text").map(|t| match t.into() {
                        TermRef::Literal(l) => l.value().to_string(),
                        _ => String::new(),
                    })
                })
                .filter(|s| !s.is_empty())
                .collect();

            if rules.is_empty() {
                format_toml_rules(domain_def)
            } else {
                let mut out = format!("[FILE MATCH: {}]\n", domain_def.name);
                for (i, rule) in rules.iter().enumerate() {
                    out.push_str(&format!("  {i}. {rule}\n"));
                }
                out
            }
        }
        _ => format_toml_rules(domain_def),
    }
}

fn format_toml_rules(domain_def: &domain::DomainDef) -> String {
    if domain_def.rules.is_empty() {
        return String::new();
    }
    let mut out = format!("[FILE MATCH: {}]\n", domain_def.name);
    for (i, rule) in domain_def.rules.iter().enumerate() {
        out.push_str(&format!("  {i}. {rule}\n"));
    }
    out
}

fn load_workspace_graph(cwd: &Path) -> Option<oxigraph::store::Store> {
    let base_dir = crate::config::find_workspace_base(cwd)?;
    let trig_path = base_dir.join("graph.trig");
    if !trig_path.exists() {
        return None;
    }
    crate::store::load_graph(&trig_path).ok()
}

fn extract_file_paths(event: &serde_json::Value) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(fp) = event
        .get("tool_input")
        .and_then(|ti| ti.get("file_path"))
        .and_then(|v| v.as_str())
    {
        paths.push(PathBuf::from(fp));
    }
    if let Some(fp) = event
        .get("tool_input")
        .and_then(|ti| ti.get("path"))
        .and_then(|v| v.as_str())
    {
        paths.push(PathBuf::from(fp));
    }
    paths
}
