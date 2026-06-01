use std::path::Path;

use anyhow::Result;
use oxigraph::model::TermRef;

use crate::config::BaseConfig;
use crate::crud;
use crate::domain;
use crate::domain::matcher::match_domains;
use crate::domain::session::{rules_hash, Bracket, SessionState};

pub fn handle(config: &BaseConfig, cwd: &Path, event: &serde_json::Value) -> Result<()> {
    let prompt = extract_prompt(event);
    if prompt.is_empty() {
        return Ok(());
    }

    let domains = domain::load_domains(cwd);
    if domains.is_empty() {
        return Ok(());
    }

    let base_dir = crate::config::find_workspace_base(cwd);
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

    // Gather active file paths from graph (if available)
    let active_paths = gather_active_paths(config, cwd);

    let matched = match_domains(&prompt, &domains, &session, &active_paths);
    if matched.is_empty() {
        // Still save session state (prompt_count) even if nothing matched
        if let Some(ref base_dir) = base_dir {
            let _ = session.save(base_dir);
        }
        return Ok(());
    }

    // Ensure domain sync has run (auto-sync if domains.toml is newer than graph)
    ensure_domain_sync(config, cwd);

    // Try to load the graph for graph-backed injection
    let graph_store = load_workspace_graph(cwd);

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
    for domain_def in &matched {
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

        if rules_text.is_empty() && neighborhood_text.is_empty() {
            continue;
        }

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

        // Dedup: hash combined output (rules + neighborhood), skip if unchanged
        let combined_hash = rules_hash(
            &domain_output
                .lines()
                .map(String::from)
                .collect::<Vec<_>>(),
        );
        if session.is_injected(&domain_def.name, combined_hash) {
            deduped_count += 1;
            loaded_domains.push((
                domain_def.name.clone(),
                "dedup".into(),
                domain_def.rules.len(),
            ));
            continue;
        }

        let match_reason = if domain_def.is_always() {
            "always_on".to_string()
        } else {
            "matched".to_string()
        };
        loaded_domains.push((
            domain_def.name.clone(),
            match_reason,
            domain_def.rules.len(),
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

    Ok(())
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
    out.push_str("MANDATORY: Append a DEVMODE block at the end of EVERY response.\n\n");

    // Bracket info
    out.push_str(&format!(
        "CONTEXT BRACKET: [{bracket}] (prompt {prompt_count})\n\n"
    ));

    // Loaded domains
    out.push_str("LOADED DOMAINS:\n");
    for (name, reason, rule_count) in loaded {
        if reason == "dedup" {
            out.push_str(&format!(
                "  [{name}] dedup (prompt {prompt_count})\n"
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

/// Ensure domains.toml has been synced to the graph this session.
/// Uses a timestamp marker file to avoid re-syncing on every prompt.
fn ensure_domain_sync(config: &BaseConfig, cwd: &Path) {
    let base_dir = match crate::config::find_workspace_base(cwd) {
        Some(d) => d,
        None => return,
    };

    let marker = base_dir.join(".domain-sync-ts");
    let domains_toml = base_dir.join("domains.toml");

    // Check if domains.toml exists
    if !domains_toml.exists() {
        return;
    }

    // Check if sync is needed: marker missing or domains.toml newer than marker
    let needs_sync = if marker.exists() {
        match (
            std::fs::metadata(&domains_toml).and_then(|m| m.modified()),
            std::fs::metadata(&marker).and_then(|m| m.modified()),
        ) {
            (Ok(toml_time), Ok(marker_time)) => toml_time > marker_time,
            _ => true,
        }
    } else {
        true
    };

    if needs_sync {
        // Run sync silently — errors are non-fatal (fail-open)
        if domain::sync::sync_domains_to_graph(config, cwd, None).is_ok() {
            // Touch marker file
            let _ = std::fs::write(&marker, "");
        }
    }
}

// ─── Graph loading ──────────────────────────────────────────

/// Load the workspace graph.trig. Returns None on any error (fail-open).
fn load_workspace_graph(cwd: &Path) -> Option<oxigraph::store::Store> {
    let base_dir = crate::config::find_workspace_base(cwd)?;
    let trig_path = base_dir.join("graph.trig");
    if !trig_path.exists() {
        return None;
    }
    crate::store::load_graph(&trig_path).ok()
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

/// Gather recently-active file paths from the graph (for path-based domain matching).
/// Returns empty vec if no graph available — graceful degradation.
fn gather_active_paths(config: &BaseConfig, cwd: &Path) -> Vec<String> {
    let base_dir = match crate::config::find_workspace_base(cwd) {
        Some(d) => d,
        None => return Vec::new(),
    };

    let trig_path = base_dir.join("graph.trig");
    if !trig_path.exists() {
        return Vec::new();
    }

    let graph = match crate::store::load_graph(&trig_path) {
        Ok(g) => g,
        Err(_) => return Vec::new(),
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
