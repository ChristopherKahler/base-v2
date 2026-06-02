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

    // Step 5: Seed system rules
    seed_system_rules(&global_dir)?;

    // Step 6: Append BASE CLI section to ~/.claude/CLAUDE.md
    let claude_md = home.join(".claude").join("CLAUDE.md");
    append_claude_md(&claude_md)?;

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

// ─── Uninstall ──────────────────────────────────────────────

/// Remove base hooks from settings.json, remove binary, strip CLAUDE.md section.
/// With --purge, also removes ~/.base-gbl/ global tier.
pub fn uninstall(purge: bool) -> Result<()> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;

    println!("═══════════════════════════════════════");
    println!("BASE v2 — Uninstall");
    println!("═══════════════════════════════════════\n");

    // 1. Remove hooks from settings.json
    let settings_path = home.join(".claude").join("settings.json");
    remove_hooks(&settings_path)?;

    // 2. Remove BASE CLI section from CLAUDE.md
    let claude_md = home.join(".claude").join("CLAUDE.md");
    remove_claude_md_section(&claude_md)?;

    // 3. Remove binary
    let binary = home.join(".local").join("bin").join("base");
    if binary.exists() {
        print!("3. Remove binary ... ");
        std::fs::remove_file(&binary)?;
        println!("✓ removed {}", binary.display());
    } else {
        println!("3. Binary not found at {} — skipped", binary.display());
    }

    // 4. Purge global tier if requested
    if purge {
        let global_dir = home.join(".base-gbl");
        if global_dir.exists() {
            print!("4. Purge global tier ... ");
            std::fs::remove_dir_all(&global_dir)?;
            println!("✓ removed {}", global_dir.display());
        }
    } else {
        println!("4. Global tier preserved (~/.base-gbl/) — use --purge to remove");
    }

    println!("\n═══════════════════════════════════════");
    println!("✓ Uninstall complete");
    println!("═══════════════════════════════════════\n");
    println!("Workspace .base/ directories are untouched.");
    println!("Remove them manually if needed: rm -rf <workspace>/.base/");

    Ok(())
}

fn remove_hooks(settings_path: &Path) -> Result<()> {
    print!("1. Remove hooks from settings.json ... ");

    if !settings_path.exists() {
        println!("not found — skipped");
        return Ok(());
    }

    let content = std::fs::read_to_string(settings_path)?;
    if !content.contains("base hook") {
        println!("no base hooks found — skipped");
        return Ok(());
    }

    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for (_event, entries) in hooks.iter_mut() {
            if let Some(arr) = entries.as_array_mut() {
                arr.retain(|entry| {
                    // Remove any entry whose hooks array contains a "base hook" command
                    if let Some(hook_list) = entry.get("hooks").and_then(|h| h.as_array()) {
                        !hook_list.iter().any(|h| {
                            h.get("command")
                                .and_then(|c| c.as_str())
                                .map(|c| c.contains("base hook"))
                                .unwrap_or(false)
                        })
                    } else {
                        true
                    }
                });
            }
        }
    }

    let tmp = settings_path.with_extension("json.tmp");
    let formatted = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&tmp, &formatted)?;
    std::fs::rename(&tmp, settings_path)?;

    println!("✓ removed all base hook entries");
    Ok(())
}

fn remove_claude_md_section(claude_md_path: &Path) -> Result<()> {
    print!("2. Remove BASE CLI section from CLAUDE.md ... ");

    if !claude_md_path.exists() {
        println!("not found — skipped");
        return Ok(());
    }

    let content = std::fs::read_to_string(claude_md_path)?;

    if !content.contains("## BASE CLI") {
        println!("not present — skipped");
        return Ok(());
    }

    // Find and remove the BASE CLI section (from "## BASE CLI" to end of file or next ## heading)
    let mut lines: Vec<&str> = content.lines().collect();
    let start = lines.iter().position(|l| l.starts_with("## BASE CLI"));

    if let Some(start_idx) = start {
        // Find the next ## heading after the BASE CLI section (or end of file)
        let end = lines[start_idx + 1..]
            .iter()
            .position(|l| l.starts_with("## ") && !l.starts_with("### "))
            .map(|pos| start_idx + 1 + pos)
            .unwrap_or(lines.len());

        lines.drain(start_idx..end);

        let new_content = lines.join("\n");
        let tmp = claude_md_path.with_extension("md.tmp");
        std::fs::write(&tmp, new_content.trim_end())?;
        std::fs::rename(&tmp, claude_md_path)?;

        println!("✓ removed");
    } else {
        println!("not found — skipped");
    }

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

    // docs/markdown-ontology-protocol.md — bundled MOP spec
    let docs_dir = global_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;
    let mop_path = docs_dir.join("markdown-ontology-protocol.md");
    if !mop_path.exists() {
        std::fs::write(&mop_path, include_str!("../docs/markdown-ontology-protocol.md"))?;
        println!("   Created docs/markdown-ontology-protocol.md");
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

// ─── Step 5: Seed system rules ──────────────────────────────

fn seed_system_rules(global_dir: &Path) -> Result<()> {
    print!("5. Seed system rules ... ");

    let config = BaseConfig::load(global_dir);

    // Sync domains first so GLOBAL domain entity exists
    let _ = crate::domain::sync::sync_domains_to_graph(&config, global_dir, None);

    // Check if MOP rule already exists
    let ns = &config.namespace;
    let p = &ns.prefix;
    let domain_iri = crate::crud::build_iri(ns, "domain", "global");

    let check = format!(
        "SELECT ?text WHERE {{ GRAPH ?g {{ <{domain_iri}> {p}:hasRule ?r . ?r {p}:ruleText ?text . FILTER(CONTAINS(?text, \"Markdown Ontology Protocol\")) }} }}"
    );

    let already_exists = if let Ok(results) = crate::crud::load_and_query(global_dir, ns, &check) {
        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            solutions.filter_map(|r| r.ok()).next().is_some()
        } else {
            false
        }
    } else {
        false
    };

    if already_exists {
        println!("✓ (already seeded)");
        return Ok(());
    }

    // Seed MOP rule
    let _ = crate::crud::rule::add(
        global_dir,
        ns,
        "GLOBAL",
        "When writing or editing markdown files, follow the Markdown Ontology Protocol (MOP) — use YAML frontmatter with type, status, tags, and relatedTo fields so base sync can extract the document into the graph. Read ~/.base-gbl/docs/markdown-ontology-protocol.md before writing frontmatter.",
    );

    println!("✓ (MOP rule added to GLOBAL)");
    Ok(())
}

// ─── Step 6: CLAUDE.md integration ──────────────────────────

const BASE_CLI_SECTION: &str = r#"
## BASE CLI — Proactive Context Engine

The `base` binary is on PATH. Use these commands proactively during sessions — they write to a knowledge graph that persists across sessions and surfaces context automatically.

### When to call (proactive, not on-demand)

| Trigger | Command |
|---------|---------|
| A decision is made (architectural, process, tooling) | `base decision log --domain X --decision "..." --rationale "..."` |
| An insight, correction, or lesson emerges | `base learn --text "..." --domain X --type insight\|correction\|decision` |
| User defines or refines a behavioral rule | `base rule add --domain X --text "..."` |
| Before making assumptions about prior context | `base recall --keyword "..."` or `base recall --domain X` |
| User asks to scaffold a new workspace | `base scaffold [path]` |

### Commands reference

- `base learn --text "..." [--domain X] [--type insight]` — structured memory with relational edges
- `base recall --keyword "..." [--domain X]` — graph-backed relational search
- `base rule add --domain X --text "..."` — add a rule to a domain (graph-native)
- `base rule list --domain X` — show rules for a domain
- `base rule remove --domain X --index N` — remove a rule
- `base decision log --domain X --decision "..." --rationale "..."` — log a decision
- `base decision search --keyword "..."` — find prior decisions
- `base project add --name "..." --status active` — register a project
- `base task add --project X --name "..."` — add a task
- `base scaffold [path]` — set up a new workspace

### What happens automatically (via hooks)

- **Session start:** Graph syncs domains, ingests paul.toml projects, runs signals
- **User prompt:** Matches keywords → injects domain rules + decisions + notes from graph
- **Pre-tool-use:** Matches file paths → injects relevant domain rules before tool executes
- **Post-tool-use:** Updates lastActive timestamps in graph

### Architecture

- Rules, decisions, notes, and projects are graph entities with relational edges
- `domains.toml` defines triggers only (keywords, paths) — rule content lives in the graph
- `~/.base-gbl/` = global tier, `{workspace}/.base/` = workspace tier
- Built by Chris Kahler · Chris AI Systems · https://chrisai.cv/skool
"#;

fn append_claude_md(claude_md_path: &Path) -> Result<()> {
    print!("5. CLAUDE.md integration ... ");

    if !claude_md_path.exists() {
        std::fs::write(claude_md_path, BASE_CLI_SECTION.trim_start())?;
        println!("✓ (created with BASE CLI section)");
        return Ok(());
    }

    let content = std::fs::read_to_string(claude_md_path)?;

    if content.contains("## BASE CLI") {
        println!("already present");
        return Ok(());
    }

    let mut new_content = content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(BASE_CLI_SECTION);

    let tmp = claude_md_path.with_extension("md.tmp");
    std::fs::write(&tmp, &new_content)?;
    std::fs::rename(&tmp, claude_md_path)?;

    println!("✓ (appended BASE CLI section)");
    Ok(())
}
