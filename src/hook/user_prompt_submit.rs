use std::path::Path;

use anyhow::Result;
use oxigraph::model::TermRef;

use crate::config::BaseConfig;
use crate::crud;
use crate::domain;
use crate::domain::matcher::match_domains;
use crate::domain::session::{rules_hash, Bracket, SessionState};

pub fn handle(config: &BaseConfig, cwd: &Path, event: &serde_json::Value) -> Result<super::HookEventData> {
    let prompt = extract_prompt(event);
    if prompt.is_empty() {
        return Ok(super::HookEventData::default());
    }

    let domains = domain::load_domains(cwd);
    if domains.is_empty() {
        return Ok(super::HookEventData::default());
    }

    // Resolve base dir: workspace first, fall back to global tier
    let base_dir = crate::config::find_workspace_base(cwd)
        .or_else(|| {
            dirs::home_dir().map(|h| h.join(".base-gbl").join(".base")).filter(|p| p.is_dir())
        });
    let mut session = base_dir
        .as_deref()
        .map(SessionState::load)
        .unwrap_or_default();

    // Track prompt count and derive bracket
    session.increment_prompt();
    let bracket = session.bracket(&config.bracket);

    // Force-refresh dedup in DEPLETED/CRITICAL on interval
    if session.should_force_refresh(&config.bracket) {
        session.clear_dedup();
    }

    // Check for *COMMAND before domain matching
    let commands = crate::command::load_commands(cwd);
    if let Some(cmd) = crate::command::match_command(&prompt, &commands) {
        let cmd_output = crate::command::format_command_output(cmd);
        if !cmd_output.is_empty() {
            // Star commands bypass domain matching — they're explicit invocations
            if let Some(ref base_dir) = base_dir {
                let _ = session.save(base_dir);
            }
            print!("{cmd_output}");
            return Ok(super::HookEventData {
                prompt_num: Some(session.prompt_count),
                ..Default::default()
            });
        }
    }

    // Gather active file paths from graph (if available)
    let active_paths = gather_active_paths(config, cwd);

    let matched = match_domains(&prompt, &domains, &session, &active_paths);
    if matched.is_empty() {
        // Still save session state (prompt_count) even if nothing matched
        if let Some(ref base_dir) = base_dir {
            let _ = session.save(base_dir);
        }
        return Ok(super::HookEventData {
            prompt_num: Some(session.prompt_count),
            ..Default::default()
        });
    }

    // Ensure domain sync has run (auto-sync if domains.toml is newer than graph)
    ensure_domain_sync(config, cwd);

    // Try to load the graph for graph-backed injection (merged: global + workspace)
    let graph_store = load_merged_graph(cwd);

    // Emit context bracket tag
    let mut output = format!(
        "<context-bracket>[{}] (prompt {})</context-bracket>\n\n",
        bracket, session.prompt_count
    );

    // Determine if we're in lean mode (FRESH, first 2 prompts — rules only, skip neighborhood)
    let lean_mode = bracket == Bracket::Fresh && session.prompt_count <= 2;

    // Track injection metadata for DEVMODE
    let mut loaded_domains: Vec<(String, String, usize)> = Vec::new(); // (name, match_reason, rule_count)
    let mut deduped_count = 0usize;

    // Format and emit matched rules
    for dm in &matched {
        let domain_def = dm.domain;

        // Try graph-backed injection first, fall back to TOML rules
        let (rules_text, neighborhood_text) = match &graph_store {
            Some(store) => {
                let (r, n) = query_domain_from_graph(store, config, domain_def);
                if lean_mode {
                    (r, String::new()) // skip neighborhood in lean mode
                } else {
                    (r, n)
                }
            }
            None => (format_toml_rules(domain_def), String::new()),
        };

        // Don't short-circuit if domain has a query — query-only domains have no rules/neighborhood
        if rules_text.is_empty() && neighborhood_text.is_empty() && domain_def.query.is_none() {
            continue;
        }

        // Query notes linked to this domain (skip in lean mode)
        let notes_text = if lean_mode {
            String::new()
        } else if let Some(ref store) = graph_store {
            let domain_slug = crud::slugify(&domain_def.name);
            let domain_iri = crud::build_iri(&config.namespace, "domain", &domain_slug);
            let notes = crate::crud::note::notes_for_domain(store, &config.namespace, &domain_iri);
            if notes.is_empty() {
                String::new()
            } else {
                let mut out = format!("[{} NOTES]\n", domain_def.name);
                for (note_type, text) in &notes {
                    out.push_str(&format!("  - {note_type}: {text}\n"));
                }
                out
            }
        } else {
            String::new()
        };

        // Query-triggered injection: if domain has a `query` field, run the external SPARQL file
        let query_text = match (&graph_store, &domain_def.query) {
            (Some(store), Some(query_name)) => {
                let fmt = domain_def.query_format.as_deref().unwrap_or("list");
                resolve_and_run_query(store, config, cwd, query_name, fmt, &domain_def.name)
            }
            _ => String::new(),
        };

        // Build combined output for this domain
        let mut domain_output = String::new();
        if !rules_text.is_empty() {
            domain_output.push_str(&rules_text);
        }
        if !neighborhood_text.is_empty() {
            if !domain_output.is_empty() {
                domain_output.push('\n');
            }
            domain_output.push_str(&neighborhood_text);
        }
        if !notes_text.is_empty() {
            if !domain_output.is_empty() {
                domain_output.push('\n');
            }
            domain_output.push_str(&notes_text);
        }
        if !query_text.is_empty() {
            if !domain_output.is_empty() {
                domain_output.push('\n');
            }
            domain_output.push_str(&query_text);
        }

        // Dedup: hash combined output (rules + neighborhood), skip if unchanged
        let combined_hash = rules_hash(
            &domain_output
                .lines()
                .map(String::from)
                .collect::<Vec<_>>(),
        );
        // Count actual injected rules (from graph, not TOML)
        let injected_rule_count = rules_text.lines().filter(|l| l.starts_with("  ")).count();

        if session.is_injected(&domain_def.name, combined_hash) {
            deduped_count += 1;
            let dedup_reason = if config.devmode.enabled {
                format!("dedup [{}]", dm.reason)
            } else {
                "dedup".into()
            };
            loaded_domains.push((
                domain_def.name.clone(),
                dedup_reason,
                injected_rule_count,
            ));
            continue;
        }

        // Use the actual match reason from the matcher (only meaningful in DEVMODE)
        let match_reason = if config.devmode.enabled {
            format!("{}", dm.reason)
        } else if domain_def.is_always() {
            "always_on".to_string()
        } else {
            "matched".to_string()
        };
        loaded_domains.push((
            domain_def.name.clone(),
            match_reason,
            injected_rule_count,
        ));

        output.push_str(&domain_output);
        output.push('\n');

        // Mark as injected in session state
        session.mark_injected(&domain_def.name, combined_hash);

        // Track sticky domains
        if domain_def.sticky {
            session.mark_injected(&domain_def.name, combined_hash);
        }
    }

    // DEVMODE block (Task 2 will populate this fully)
    if config.devmode.enabled {
        output.push_str(&format_devmode_block(
            &loaded_domains,
            &domains,
            bracket,
            session.prompt_count,
            deduped_count,
        ));
    }

    // Save updated session state
    if let Some(ref base_dir) = base_dir {
        let _ = session.save(base_dir);
    }

    if !output.is_empty() {
        print!("{}", output.trim_end());
    }

    // Build event data for JSONL logging
    let domains_matched: Vec<String> = loaded_domains
        .iter()
        .filter(|(_, reason, _)| !reason.starts_with("dedup"))
        .map(|(name, _, _)| name.clone())
        .collect();
    let total_rules: usize = loaded_domains
        .iter()
        .filter(|(_, reason, _)| !reason.starts_with("dedup"))
        .map(|(_, _, count)| count)
        .sum();

    // Capture first 120 chars of the prompt for dashboard display
    let prompt_preview = if prompt.len() > 120 {
        Some(format!("{}…", &prompt[..117]))
    } else {
        Some(prompt.clone())
    };

    Ok(super::HookEventData {
        domains_matched,
        rules_injected: total_rules,
        suppressed: deduped_count,
        prompt_num: Some(session.prompt_count),
        prompt_text: prompt_preview,
        tool_name: None,
        file_path: None,
        session_id: None, // populated by run() after handle returns
        ..Default::default()
    })
}

// ─── DEVMODE output ─────────────────────────────────────────

/// Format the DEVMODE instruction block for Claude.
pub fn format_devmode_block(
    loaded: &[(String, String, usize)],
    all_domains: &[domain::DomainDef],
    bracket: Bracket,
    prompt_count: u32,
    deduped: usize,
) -> String {
    let mut out = String::new();
    out.push_str("\n⚠️ DEVMODE=true ⚠️\n");
    out.push_str("============================================================\n");
    out.push_str("MANDATORY: Append a DEVMODE block at the end of EVERY response.\n");
    out.push_str("NEVER skip it. NEVER forget it. NEVER omit it for any reason.\n");
    out.push_str("NEVER fabricate data in the block — only report what you actually received.\n\n");
    out.push_str("Format EXACTLY (keep under 8 lines, no rationale, no prose):\n");
    out.push_str("---\n```\n");
    out.push_str("🔧 DEVMODE\n");
    out.push_str("Bracket: [X] (prompt N)\n");
    out.push_str("Loaded: domain1 [reason] (N rules), domain2 [reason] (dedup)\n");
    out.push_str("Available: domain3, domain4, ...\n");
    out.push_str("Dedup: N skipped\n");
    out.push_str("Tools: tools used this response, or 'none'\n");
    out.push_str("```\n---\n");
    out.push_str("============================================================\n\n");

    // Bracket info
    out.push_str(&format!(
        "CONTEXT BRACKET: [{bracket}] (prompt {prompt_count})\n\n"
    ));

    // Loaded domains
    out.push_str("LOADED DOMAINS:\n");
    for (name, reason, rule_count) in loaded {
        if reason.starts_with("dedup") {
            out.push_str(&format!(
                "  [{name}] {reason} (prompt {prompt_count})\n"
            ));
        } else {
            out.push_str(&format!(
                "  [{name}] {reason} ({rule_count} rules)\n"
            ));
        }
    }

    // Available (not loaded) domains
    let loaded_names: Vec<&str> = loaded.iter().map(|(n, _, _)| n.as_str()).collect();
    let available: Vec<&domain::DomainDef> = all_domains
        .iter()
        .filter(|d| !loaded_names.contains(&d.name.as_str()) && !d.is_always())
        .collect();

    if !available.is_empty() {
        out.push_str("\nAVAILABLE (not loaded):\n");
        for d in &available {
            let kws = d.prompt_keywords.join(", ");
            out.push_str(&format!("  {} ({})\n", d.name, kws));
        }
    }

    if deduped > 0 {
        out.push_str(&format!("\nDEDUP: {deduped} domain(s) skipped (unchanged)\n"));
    }

    out
}

// ─── Graph-backed injection ─────────────────────────────────

/// Query a domain's rules and 1-hop neighborhood from the graph.
/// Returns (rules_text, neighborhood_text). Falls back to TOML if graph query fails.
fn query_domain_from_graph(
    store: &oxigraph::store::Store,
    config: &BaseConfig,
    domain_def: &domain::DomainDef,
) -> (String, String) {
    let ns = &config.namespace;
    let p = &ns.prefix;
    let domain_slug = crud::slugify(&domain_def.name);
    let domain_iri = crud::build_iri(ns, "domain", &domain_slug);
    let pfx = crud::prefixes(ns);

    // Query 1: Get rules ordered by priority
    let rules_sparql = format!(
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

    let rules_text = match crate::store::query(store, &rules_sparql) {
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
                // Fallback to TOML rules
                format_toml_rules(domain_def)
            } else {
                let mut out = format!("[DOMAIN: {}]\n", domain_def.name);
                for (i, rule) in rules.iter().enumerate() {
                    out.push_str(&format!("  {i}. {rule}\n"));
                }
                out
            }
        }
        _ => format_toml_rules(domain_def),
    };

    // Query 2: 1-hop neighborhood (decisions linked to this domain, projects with hasDomain)
    let neighborhood_sparql = format!(
        "{pfx}\n\
         SELECT ?name ?type WHERE {{\n\
           GRAPH ?g {{\n\
             {{\n\
               <{domain_iri}> {p}:hasDecision ?related .\n\
               ?related {p}:name ?name .\n\
               BIND({p}:Decision AS ?type)\n\
             }} UNION {{\n\
               ?related {p}:hasDomain <{domain_iri}> ;\n\
                 a {p}:Project ;\n\
                 {p}:name ?name .\n\
               BIND({p}:Project AS ?type)\n\
             }}\n\
           }}\n\
         }}"
    );

    let neighborhood_text = match crate::store::query(store, &neighborhood_sparql) {
        Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) => {
            let neighbors: Vec<(String, String)> = solutions
                .filter_map(|r| r.ok())
                .filter_map(|row| {
                    let name = row.get("name").map(|t| match t.into() {
                        TermRef::Literal(l) => l.value().to_string(),
                        _ => String::new(),
                    })?;
                    let type_label = row.get("type").map(|t| crud::term_display(t.into()))?;
                    if name.is_empty() {
                        None
                    } else {
                        Some((type_label, name))
                    }
                })
                .collect();

            if neighbors.is_empty() {
                String::new()
            } else {
                let mut out = format!("[{} CONTEXT]\n", domain_def.name);
                for (type_label, name) in &neighbors {
                    out.push_str(&format!("  - {type_label}: {name}\n"));
                }
                out
            }
        }
        _ => String::new(),
    };

    (rules_text, neighborhood_text)
}

// ─── Query-triggered injection ─────────────────────────────

/// Resolve a query name to a `.sparql` file, read it, run it, format results.
/// Resolution: workspace `.base/queries/{name}.sparql` → global `~/.base-gbl/queries/{name}.sparql`.
pub fn resolve_and_run_query(
    store: &oxigraph::store::Store,
    config: &BaseConfig,
    cwd: &Path,
    query_name: &str,
    format: &str,
    domain_name: &str,
) -> String {
    let filename = format!("{query_name}.sparql");

    // Tier 1: workspace
    let sparql_content = crate::config::find_workspace_base(cwd)
        .and_then(|base| std::fs::read_to_string(base.join("queries").join(&filename)).ok())
        // Tier 2: global
        .or_else(|| {
            dirs::home_dir().and_then(|home| {
                std::fs::read_to_string(home.join(".base-gbl").join("queries").join(&filename)).ok()
            })
        });

    let sparql_raw = match sparql_content {
        Some(s) => s,
        None => {
            eprintln!("base: query file not found: queries/{filename}");
            return String::new();
        }
    };

    // Prefix substitution (same pattern as queries.toml)
    let p = &config.namespace.prefix;
    let u = &config.namespace.uri;
    let sparql = sparql_raw
        .replace("{{prefix}}", p)
        .replace("{{uri}}", u);

    match crate::store::query(store, &sparql) {
        Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) => {
            let rows: Vec<_> = solutions.filter_map(|r| r.ok()).collect();
            if rows.is_empty() {
                return String::new();
            }

            let mut out = format!("<base-query name=\"{query_name}\" domain=\"{domain_name}\">\n");

            let known_vars = ["label", "name", "text", "detail", "type", "status", "value", "count", "created"];

            match format {
                "table" => {
                    if let Some(first) = rows.first() {
                        let vars: Vec<&str> = known_vars.iter()
                            .filter(|v| first.get(**v).is_some())
                            .copied()
                            .collect();

                        if !vars.is_empty() {
                            out.push_str(&format!("| {} |\n", vars.join(" | ")));
                            out.push_str(&format!("|{}|\n", vars.iter().map(|_| "---").collect::<Vec<_>>().join("|")));
                            for row in &rows {
                                let vals: Vec<String> = vars.iter()
                                    .map(|v| row.get(*v).map(|t| crud::term_display(t.into())).unwrap_or_default())
                                    .collect();
                                out.push_str(&format!("| {} |\n", vals.join(" | ")));
                            }
                        }
                    }
                }
                "prose" => {
                    for row in &rows {
                        for var in &known_vars[..7] {
                            if let Some(term) = row.get(*var) {
                                let val = crud::term_display(term.into());
                                if !val.is_empty() {
                                    out.push_str(&format!("{var}: {val}\n"));
                                }
                            }
                        }
                        out.push('\n');
                    }
                }
                _ => {
                    // Default: "list"
                    for row in &rows {
                        let primary = row.get("label")
                            .or_else(|| row.get("name"))
                            .or_else(|| row.get("text"))
                            .map(|t| crud::term_display(t.into()))
                            .unwrap_or_default();
                        let detail = row.get("detail")
                            .or_else(|| row.get("value"))
                            .map(|t| crud::term_display(t.into()));

                        if let Some(d) = detail {
                            out.push_str(&format!("- {primary}: {d}\n"));
                        } else {
                            out.push_str(&format!("- {primary}\n"));
                        }
                    }
                }
            }

            out.push_str("</base-query>\n");
            out
        }
        Ok(_) => String::new(),
        Err(e) => {
            eprintln!("base: query '{query_name}' failed: {e}");
            String::new()
        }
    }
}

/// Format rules directly from the DomainDef struct (TOML fallback).
fn format_toml_rules(domain_def: &domain::DomainDef) -> String {
    if domain_def.rules.is_empty() {
        return String::new();
    }
    let mut out = format!("[DOMAIN: {}]\n", domain_def.name);
    for (i, rule) in domain_def.rules.iter().enumerate() {
        out.push_str(&format!("  {i}. {rule}\n"));
    }
    out
}

// ─── Auto-sync ──────────────────────────────────────────────

/// Public wrapper for pre_tool_use to call.
pub fn ensure_domain_sync_pub(config: &BaseConfig, cwd: &Path) {
    ensure_domain_sync(config, cwd);
}

/// Ensure domains.toml has been synced to the graph this session.
/// Uses a timestamp marker file to avoid re-syncing on every prompt.
/// Syncs both global (~/.base-gbl/) and workspace tiers.
fn ensure_domain_sync(config: &BaseConfig, cwd: &Path) {
    // Global tier: sync ~/.base-gbl/domains.toml → ~/.base-gbl/.base/graph.nq
    if let Some(home) = dirs::home_dir() {
        let global_dir = home.join(".base-gbl");
        let global_base = global_dir.join(".base");
        if global_base.is_dir() {
            let marker = global_base.join(".domain-sync-ts");
            let domains_toml = global_dir.join("domains.toml");
            if domains_toml.exists() {
                let needs_sync = needs_sync_check(&domains_toml, &marker);
                if needs_sync {
                    if domain::sync::sync_domains_to_graph(config, &global_dir, None).is_ok() {
                        let _ = std::fs::write(&marker, "");
                    }
                }
            }
        }
    }

    // Workspace tier: sync {workspace}/.base/domains.toml → {workspace}/.base/graph.nq
    let base_dir = match crate::config::find_workspace_base(cwd) {
        Some(d) => d,
        None => return,
    };

    let marker = base_dir.join(".domain-sync-ts");
    let domains_toml = base_dir.join("domains.toml");

    if !domains_toml.exists() {
        return;
    }

    let needs_sync = needs_sync_check(&domains_toml, &marker);
    if needs_sync {
        if domain::sync::sync_domains_to_graph(config, cwd, None).is_ok() {
            let _ = std::fs::write(&marker, "");
        }
    }
}

/// Check if a domains.toml is newer than its sync marker.
fn needs_sync_check(domains_toml: &Path, marker: &Path) -> bool {
    if marker.exists() {
        match (
            std::fs::metadata(domains_toml).and_then(|m| m.modified()),
            std::fs::metadata(marker).and_then(|m| m.modified()),
        ) {
            (Ok(toml_time), Ok(marker_time)) => toml_time > marker_time,
            _ => true,
        }
    } else {
        true
    }
}

// ─── Graph loading ──────────────────────────────────────────

/// Load a merged graph from global (~/.base-gbl/.base/graph.nq) and workspace tiers.
/// Both are loaded into one Oxigraph store so SPARQL queries span all tiers.
/// Returns None only if neither graph exists (fail-open).
fn load_merged_graph(cwd: &Path) -> Option<oxigraph::store::Store> {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();

    // Global tier: ~/.base-gbl/.base/graph.nq
    if let Some(home) = dirs::home_dir() {
        let global_trig = home.join(".base-gbl").join(".base").join("graph.nq");
        if global_trig.exists() {
            paths.push(global_trig);
        }
    }

    // Workspace tier: {workspace}/.base/graph.nq
    if let Some(base_dir) = crate::config::find_workspace_base(cwd) {
        let ws_trig = base_dir.join("graph.nq");
        if ws_trig.exists() {
            paths.push(ws_trig);
        }
    }

    if paths.is_empty() {
        return None;
    }

    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    crate::store::load_graphs(&path_refs).ok()
}

// ─── Prompt extraction ──────────────────────────────────────

/// Extract prompt text from the hook event JSON.
fn extract_prompt(event: &serde_json::Value) -> String {
    // Claude Code UserPromptSubmit sends prompt in various locations
    event
        .get("prompt")
        .and_then(|v| v.as_str())
        .or_else(|| {
            event
                .get("tool_input")
                .and_then(|ti| ti.get("prompt"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("")
        .to_string()
}

/// Gather recently-active file paths from the merged graph (for path-based domain matching).
/// Returns empty vec if no graph available — graceful degradation.
fn gather_active_paths(config: &BaseConfig, cwd: &Path) -> Vec<String> {
    let graph = match load_merged_graph(cwd) {
        Some(g) => g,
        None => return Vec::new(),
    };

    let sparql = format!(
        "PREFIX {p}: <{u}>\n\
         SELECT ?path WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity {p}:path ?path .\n\
             ?entity {p}:lastActive ?ts .\n\
           }}\n\
         }}",
        p = config.namespace.prefix,
        u = config.namespace.uri,
    );

    match crate::store::query(&graph, &sparql) {
        Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) => solutions
            .filter_map(|r| r.ok())
            .filter_map(|row| {
                row.get("path")
                    .map(|t| match t.into() {
                        TermRef::Literal(l) => l.value().to_string(),
                        _ => String::new(),
                    })
                    .filter(|s| !s.is_empty())
            })
            .collect(),
        _ => Vec::new(),
    }
}
