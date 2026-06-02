use std::path::{Path, PathBuf};

use anyhow::Result;
use oxigraph::model::TermRef;

use crate::config::BaseConfig;
use crate::crud;
use crate::domain;
use crate::domain::session::SessionState;

/// PreToolUse: see file path in tool call → match file_keywords + path triggers → inject rules BEFORE tool executes.
/// Also: inject AST file map for source files, and redirect grep/find to ast query.
pub fn handle(config: &BaseConfig, cwd: &Path, event: &serde_json::Value) -> Result<()> {
    let mut output = String::new();

    // ─── Grep/find intercept (Bash tool) ─────────────────────
    if let Some(hint) = grep_intercept(event) {
        output.push_str(&hint);
        output.push('\n');
    }

    // ─── Domain rule injection (file path match) ─────────────
    let file_paths = extract_file_paths(event);
    if !file_paths.is_empty() {
        let domains = domain::load_domains(cwd);
        if !domains.is_empty() {
            let base_dir = crate::config::find_workspace_base(cwd);
            let session = base_dir
                .as_deref()
                .map(SessionState::load)
                .unwrap_or_default();

            let file_path_strings: Vec<String> = file_paths
                .iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect();

            let matched = match_by_file(&domains, &file_path_strings, &session);
            if !matched.is_empty() {
                crate::hook::user_prompt_submit::ensure_domain_sync_pub(config, cwd);
                let graph_store = load_workspace_graph(cwd);

                for domain_def in &matched {
                    let rules_text = match &graph_store {
                        Some(store) => query_rules_from_graph(store, config, domain_def),
                        None => format_toml_rules(domain_def),
                    };
                    if !rules_text.is_empty() {
                        output.push_str(&rules_text);
                        output.push('\n');
                    }
                }
            }
        }

        // ─── AST file map injection (source files) ──────────────
        for fp in &file_paths {
            if let Some(fp_str) = fp.to_str() {
                if is_source_file(fp_str) {
                    // Session dedup: only inject AST map once per file per session
                    let base_dir = crate::config::find_workspace_base(cwd);
                    let mut session = base_dir
                        .as_deref()
                        .map(SessionState::load)
                        .unwrap_or_default();

                    if !session.has_ast_injected(fp_str) {
                        if let Some(map) = crud::ast_query::file_map_compact(cwd, &config.namespace, fp_str) {
                            output.push_str(&map);
                            output.push('\n');
                            session.mark_ast_injected(fp_str);
                            if let Some(bd) = base_dir.as_deref() {
                                let _ = session.save(bd);
                            }
                        }
                    }
                }
            }
        }
    }

    if !output.is_empty() {
        print!("{}", output.trim_end());
    }

    Ok(())
}

/// Check if a file path is a source code file worth AST injection.
fn is_source_file(path: &str) -> bool {
    let exts = [
        ".rs", ".py", ".js", ".ts", ".go", ".jsx", ".tsx", ".c", ".cpp", ".h", ".hpp",
        ".java", ".rb", ".swift", ".kt", ".kts", ".scala", ".php", ".cs", ".lua", ".zig",
        ".ps1", ".ex", ".exs", ".jl", ".vue", ".svelte", ".astro", ".dart", ".sql", ".r",
        ".f90", ".pas", ".sh", ".bash", ".json", ".toml", ".yaml", ".yml",
    ];
    exts.iter().any(|ext| path.ends_with(ext))
}

/// Detect grep/find/rg in Bash commands and suggest ast query instead.
fn grep_intercept(event: &serde_json::Value) -> Option<String> {
    let tool_name = event.get("tool_name").and_then(|v| v.as_str())?;
    if tool_name != "Bash" {
        return None;
    }

    let command = event
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|v| v.as_str())?;

    // Intercept code search patterns (grep, rg, ag, ack, fd, find)
    let is_code_search = command.starts_with("grep -r")
        || command.starts_with("grep -rn")
        || command.starts_with("grep -n")
        || command.starts_with("grep -l")
        || command.starts_with("grep -rl")
        || command.contains("| grep")
        || command.starts_with("rg ")
        || command.starts_with("ag ")
        || command.starts_with("ack ")
        || command.starts_with("fd ")
        || (command.starts_with("find ") && command.contains("-name"));

    if !is_code_search {
        return None;
    }

    // Try to extract the search term
    let search_term = extract_search_term(command);

    let suggestion = if let Some(term) = search_term {
        format!(
            "<ast-hint>\n\
             AST graph available for this workspace. Try:\n\
               base ast query --contains \"{term}\"\n\
             The graph knows file locations, line numbers, and call relationships.\n\
             </ast-hint>"
        )
    } else {
        "<ast-hint>\n\
         AST graph available for this workspace. Try `base ast query` for code navigation.\n\
         Modes: --contains <name>, --file <path>, --calls <name>, --imports <path>\n\
         </ast-hint>"
            .to_string()
    };

    Some(suggestion)
}

/// Best-effort extraction of search term from grep/rg/find commands.
fn extract_search_term(command: &str) -> Option<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    // grep -r "term" or grep -rn "term"
    if parts.first().map(|s| *s == "grep").unwrap_or(false) {
        for part in parts.iter() {
            // Skip flags
            if part.starts_with('-') {
                continue;
            }
            // Skip "grep" itself
            if *part == "grep" {
                continue;
            }
            // First non-flag, non-grep token is the pattern
            let term = part.trim_matches('"').trim_matches('\'');
            if !term.is_empty() && !term.starts_with('/') && !term.starts_with('.') {
                return Some(term.to_string());
            }
        }
    }

    // rg "term"
    if parts.first().map(|s| *s == "rg").unwrap_or(false) {
        if let Some(term) = parts.get(1) {
            let t = term.trim_matches('"').trim_matches('\'');
            if !t.starts_with('-') {
                return Some(t.to_string());
            }
        }
    }

    None
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
