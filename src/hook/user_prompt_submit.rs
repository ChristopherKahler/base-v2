use std::path::Path;

use anyhow::Result;

use crate::config::BaseConfig;
use crate::domain;
use crate::domain::matcher::match_domains;
use crate::domain::session::{rules_hash, SessionState};

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

    // Gather active file paths from graph (if available)
    let active_paths = gather_active_paths(config, cwd);

    let matched = match_domains(&prompt, &domains, &session, &active_paths);
    if matched.is_empty() {
        return Ok(());
    }

    // Format and emit matched rules
    let mut output = String::new();
    for domain in &matched {
        if domain.rules.is_empty() {
            continue;
        }

        output.push_str(&format!("[DOMAIN: {}]\n", domain.name));
        for (i, rule) in domain.rules.iter().enumerate() {
            output.push_str(&format!("  {i}. {rule}\n"));
        }
        output.push('\n');

        // Mark as injected in session state
        let hash = rules_hash(&domain.rules);
        session.mark_injected(&domain.name, hash);

        // Track sticky domains
        if domain.sticky {
            session.mark_injected(&domain.name, hash);
        }
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
                    .map(|t| {
                        use oxigraph::model::TermRef;
                        match t.into() {
                            TermRef::Literal(l) => l.value().to_string(),
                            _ => String::new(),
                        }
                    })
                    .filter(|s| !s.is_empty())
            })
            .collect(),
        _ => Vec::new(),
    }
}
