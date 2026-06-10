use std::path::Path;

use anyhow::{Context, Result};

use crate::config::BaseConfig;

/// Scaffold a new workspace: create .base/, write starter configs, register globally.
pub fn run(target: &Path) -> Result<()> {
    println!("═══════════════════════════════════════");
    println!("BASE — Scaffold Workspace");
    println!("═══════════════════════════════════════\n");

    let base_dir = target.join(".base");

    // Step 1: Create .base/
    print!("1. Create .base/ ... ");
    if base_dir.exists() {
        println!("exists (preserved)");
    } else {
        std::fs::create_dir_all(&base_dir)
            .with_context(|| format!("Creating {}", base_dir.display()))?;
        println!("✓");
    }

    // Step 2: Write starter domains.toml
    let domains_path = base_dir.join("domains.toml");
    print!("2. Workspace domains.toml ... ");
    if domains_path.exists() {
        println!("exists (preserved)");
    } else {
        let ws_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace");
        std::fs::write(
            &domains_path,
            format!(
                r#"# BASE — Workspace domain triggers for {ws_name}
# Built by Chris Kahler · Chris AI Systems
# Community: https://chrisai.cv/skool
#
# Workspace-specific triggers. Rules live in the graph (base rule add/list/remove).
# Global domains in ~/.base-gbl/domains.toml apply everywhere.

# Example:
# [[domain]]
# name = "MY-PROJECT"
# mode = "triggered"
# prompt_keywords = ["my project", "working on my-project"]
# paths = ["src/my-project/"]
"#
            ),
        )?;
        println!("✓");
    }

    // Step 3: Write starter base.toml (workspace overrides)
    let config_path = base_dir.join("base.toml");
    print!("3. Workspace base.toml ... ");
    if config_path.exists() {
        println!("exists (preserved)");
    } else {
        std::fs::write(
            &config_path,
            r#"# BASE — Workspace config overrides
# Inherits from ~/.base-gbl/base.toml. Only set what you want to override.

# [namespace]
# prefix = "ops"
# uri = "http://ops-sys.local/ontology#"

# [devmode]
# enabled = true
"#,
        )?;
        println!("✓");
    }

    // Step 4: Register in ~/.base-gbl/base.toml
    print!("4. Register workspace ... ");
    let target_str = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf())
        .to_string_lossy()
        .to_string();
    register_workspace(&target_str)?;

    // Step 5: Initialize graph
    print!("5. Sync domains to graph ... ");
    let config = BaseConfig::load(target);
    match crate::domain::sync::sync_domains_to_graph(&config, target, None) {
        Ok(stats) => println!("✓ ({} domains, {} rules)", stats.domains, stats.rules),
        Err(_) => println!("✓ (empty graph initialized)"),
    }

    println!("\n═══════════════════════════════════════");
    println!("✓ Workspace scaffolded");
    println!("═══════════════════════════════════════\n");
    println!("  Path: {}", target.display());
    println!("  Config: .base/base.toml");
    println!("  Domains: .base/domains.toml");
    println!("  Graph: .base/graph.nq\n");
    println!("Next:");
    println!("  Add workspace-specific domain triggers to .base/domains.toml");
    println!("  Add rules: base rule add --domain MY-DOMAIN --text \"...\"");

    println!("\n───────────────────────────────────────");
    println!("BASE — Built by Chris Kahler");
    println!("Chris AI Systems");
    println!();
    println!("Community & support:");
    println!("  https://chrisai.cv/skool");
    println!("───────────────────────────────────────");

    Ok(())
}

/// Add a workspace path to ~/.base-gbl/base.toml [[workspace]] if not already present.
fn register_workspace(path: &str) -> Result<()> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    let global_toml = home.join(".base-gbl").join("base.toml");

    if !global_toml.exists() {
        println!("⊘ no ~/.base-gbl/base.toml (run base install first)");
        return Ok(());
    }

    let content = std::fs::read_to_string(&global_toml)?;

    // Check if already registered — exact path match on the toml value,
    // not substring (contains() would falsely match "/a/project" against
    // "/a/project-v2"; AUDIT Q11).
    let path_line = format!("path = \"{path}\"");
    if content.lines().any(|l| l.trim() == path_line) {
        println!("already registered");
        return Ok(());
    }

    // Append [[workspace]] entry
    let entry = format!("\n[[workspace]]\npath = \"{path}\"\n");
    let mut new_content = content;
    new_content.push_str(&entry);

    let tmp = global_toml.with_extension("toml.tmp");
    std::fs::write(&tmp, &new_content)?;
    std::fs::rename(&tmp, &global_toml)?;

    println!("✓ (added to ~/.base-gbl/base.toml)");
    Ok(())
}
