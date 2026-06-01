use std::path::Path;

use anyhow::{Context, Result};

use crate::config::BaseConfig;

/// Run the full install process: build, symlink, create global tier, wire hooks.
pub fn run(carl_json_path: Option<&Path>, skip_hooks: bool) -> Result<()> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    let binary_path = std::env::current_exe().context("Cannot determine binary path")?;

    println!("═══════════════════════════════════════");
    println!("BASE v2 — Global Install");
    println!("═══════════════════════════════════════\n");

    // Step 1: Copy binary to ~/.local/bin/base
    let local_bin = home.join(".local").join("bin");
    let dest_path = local_bin.join("base");
    install_binary(&binary_path, &dest_path, &local_bin)?;

    // Step 2: Create ~/.base-gbl/ with defaults
    let global_dir = home.join(".base-gbl");
    create_global_tier(&global_dir)?;

    // Step 3: Wire hooks in ~/.claude/settings.json
    if !skip_hooks {
        let settings_path = home.join(".claude").join("settings.json");
        wire_hooks(&settings_path)?;
    } else {
        println!("⊘ Hook wiring skipped (--skip-hooks)\n");
    }

    // Step 4: Migrate carl.json decisions if provided
    if let Some(carl_path) = carl_json_path {
        migrate_carl(&global_dir, carl_path)?;
    }

    println!("═══════════════════════════════════════");
    println!("✓ Install complete");
    println!("═══════════════════════════════════════\n");
    println!("Next steps:");
    println!("  1. Open a new Claude Code session");
    println!("  2. Type a prompt that matches a domain keyword");
    println!("  3. Verify rules inject from the graph\n");
    if carl_json_path.is_none() {
        println!("Optional: migrate CARL decisions:");
        println!("  base install --carl ~/.carl/carl.json\n");
    }
    println!("───────────────────────────────────────");
    println!("BASE — Built by Chris Kahler");
    println!("Chris AI Systems");
    println!();
    println!("Community & support:");
    println!("  https://chrisai.cv/skool");
    println!();
    println!("Tutorials:");
    println!("  https://www.youtube.com/@chris-ai-systems");
    println!("───────────────────────────────────────");

    Ok(())
}

// ─── Step 1: Install binary ─────────────────────────────────

fn install_binary(binary: &Path, dest: &Path, bin_dir: &Path) -> Result<()> {
    print!("1. Install binary → {} ... ", dest.display());

    std::fs::create_dir_all(bin_dir)
        .with_context(|| format!("Creating {}", bin_dir.display()))?;

    // Remove existing binary
    if dest.exists() {
        std::fs::remove_file(dest)
            .with_context(|| format!("Removing existing {}", dest.display()))?;
    }

    // Copy binary (not symlink — this is a shippable install)
    std::fs::copy(binary, dest)
        .with_context(|| format!("Copying {} → {}", binary.display(), dest.display()))?;

    // Set executable permission on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("✓");
    Ok(())
}

// ─── Step 2: Global tier ────────────────────────────────────

fn create_global_tier(global_dir: &Path) -> Result<()> {
    print!("2. Global tier → {} ... ", global_dir.display());

    std::fs::create_dir_all(global_dir)?;

    // base.toml — only create if missing
    let base_toml = global_dir.join("base.toml");
    if !base_toml.exists() {
        std::fs::write(
            &base_toml,
            r#"# BASE — Proactive context-injection engine for Claude Code
# Built by Chris Kahler · Chris AI Systems
# Community: https://chrisai.cv/skool

[namespace]
prefix = "ops"
uri = "http://ops-sys.local/ontology#"

[devmode]
enabled = true

[bracket]
enabled = true
fresh_until = 3
moderate_until = 10
depleted_until = 20
refresh_interval = 5

[signal]
enabled = true
max_chars = 2000
stale_days = 14

[sync]
include = ["**/*.md", "**/paul.json"]
exclude = ["node_modules/", "target/", ".git/", ".base/"]
"#,
        )?;
        println!("✓ (created base.toml)");
    } else {
        println!("✓ (base.toml exists, preserved)");
    }

    // domains.toml — only create if missing
    let domains_toml = global_dir.join("domains.toml");
    if !domains_toml.exists() {
        std::fs::write(
            &domains_toml,
            r#"# BASE — Domain configuration
# Built by Chris Kahler · Chris AI Systems
# Community: https://chrisai.cv/skool
#
# Global domains — loaded in every workspace.
# Workspace-specific domains go in {workspace}/.base/domains.toml

[[domain]]
name = "GLOBAL"
mode = "always"
prompt_keywords = []
file_keywords = []
rules = []
# Add your always-on rules here
"#,
        )?;
        println!("   Created domains.toml (empty — configure your domains)");
    } else {
        println!("   domains.toml exists, preserved");
    }

    Ok(())
}

// ─── Step 3: Wire hooks ─────────────────────────────────────

fn wire_hooks(settings_path: &Path) -> Result<()> {
    print!("3. Wire hooks → {} ... ", settings_path.display());

    if !settings_path.exists() {
        println!("⊘ settings.json not found, skipped");
        return Ok(());
    }

    let content = std::fs::read_to_string(settings_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse settings.json")?;

    let base_hook_command = "base hook user-prompt-submit";

    // Check if already wired
    if content.contains(base_hook_command) {
        println!("✓ (already wired)");
        return Ok(());
    }

    // Find or create UserPromptSubmit hooks array
    let hooks = settings
        .as_object_mut()
        .context("settings.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let ups_hooks = hooks
        .as_object_mut()
        .context("hooks is not an object")?
        .entry("UserPromptSubmit")
        .or_insert_with(|| serde_json::json!([]));

    let ups_array = ups_hooks
        .as_array_mut()
        .context("UserPromptSubmit is not an array")?;

    // Add base hook as a new entry
    ups_array.push(serde_json::json!({
        "hooks": [
            {
                "type": "command",
                "command": base_hook_command
            }
        ]
    }));

    // Write back atomically
    let tmp_path = settings_path.with_extension("json.tmp");
    let formatted = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&tmp_path, &formatted)?;
    std::fs::rename(&tmp_path, settings_path)?;

    println!("✓ (added base hook user-prompt-submit)");
    Ok(())
}

// ─── Step 4: Migrate CARL ───────────────────────────────────

fn migrate_carl(global_dir: &Path, carl_path: &Path) -> Result<()> {
    print!("4. Migrate CARL decisions → graph ... ");

    if !carl_path.exists() {
        println!("⊘ carl.json not found at {}", carl_path.display());
        return Ok(());
    }

    let config = BaseConfig::load(global_dir);
    match crate::domain::sync::sync_domains_to_graph(&config, global_dir, Some(carl_path)) {
        Ok(stats) => {
            println!(
                "✓ ({} domains, {} rules, {} decisions)",
                stats.domains, stats.rules, stats.decisions
            );
        }
        Err(e) => {
            println!("⚠ Migration failed: {e}");
            println!("   You can retry later: base domain sync --carl {}", carl_path.display());
        }
    }

    Ok(())
}
