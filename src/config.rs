use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

// ─── Workspace discovery ─────────────────────────────────────

/// Find the workspace `.base/` directory by walking up from cwd.
pub fn find_workspace_base(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd.to_path_buf();
    loop {
        let base = dir.join(".base");
        if base.is_dir() {
            return Some(base);
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ─── Namespace Config ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceConfig {
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default = "default_uri")]
    pub uri: String,
}

fn default_prefix() -> String {
    "ops".into()
}
fn default_uri() -> String {
    "http://ops-sys.local/ontology#".into()
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        Self {
            prefix: default_prefix(),
            uri: default_uri(),
        }
    }
}

// ─── Base Config (base.toml) ─────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BaseConfig {
    #[serde(default)]
    pub namespace: NamespaceConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub signal: SignalConfig,
}

// ─── Signal Config ───────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SignalConfig {
    #[serde(default = "default_max_chars")]
    pub max_chars: usize,
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    #[serde(default = "default_active_days")]
    pub active_days: u32,
    #[serde(default = "default_signal_enabled")]
    pub enabled: bool,
}

fn default_max_chars() -> usize { 2000 }
fn default_stale_days() -> u32 { 14 }
fn default_active_days() -> u32 { 7 }
fn default_signal_enabled() -> bool { true }

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            max_chars: default_max_chars(),
            stale_days: default_stale_days(),
            active_days: default_active_days(),
            enabled: default_signal_enabled(),
        }
    }
}

// ─── Sync Config ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
}

fn default_include() -> Vec<String> {
    vec!["**/*.md".into(), "**/paul.json".into()]
}
fn default_exclude() -> Vec<String> {
    vec![
        "node_modules/".into(),
        "target/".into(),
        ".git/".into(),
        ".base/".into(),
    ]
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            exclude: default_exclude(),
        }
    }
}

impl BaseConfig {
    /// Load config: workspace `.base/base.toml` → global `~/.base-gbl/base.toml` → compiled defaults.
    pub fn load(cwd: &Path) -> Self {
        Self::try_load(cwd).unwrap_or_default()
    }

    fn try_load(cwd: &Path) -> Option<Self> {
        let ws = cwd.join(".base").join("base.toml");
        if let Ok(c) = Self::from_file(&ws) {
            return Some(c);
        }

        let home = dirs::home_dir()?;
        let global = home.join(".base-gbl").join("base.toml");
        Self::from_file(&global).ok()
    }

    fn from_file(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
    }
}

// ─── Query Config (queries.toml) ─────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct QueryDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub sparql: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub order: u32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_format() -> String {
    "list".into()
}
fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct QueriesFile {
    #[serde(default)]
    query: Vec<QueryDef>,
}

const DEFAULT_QUERIES_TOML: &str = include_str!("queries.default.toml");

/// Load queries with tiered override: embedded defaults → global → workspace.
/// Replaces `{{prefix}}` placeholder with configured namespace prefix.
pub fn load_queries(cwd: &Path, config: &BaseConfig) -> Vec<QueryDef> {
    let mut queries = parse_queries_toml(DEFAULT_QUERIES_TOML);

    // Layer global queries
    if let Some(home) = dirs::home_dir()
        && let Ok(content) =
            std::fs::read_to_string(home.join(".base-gbl").join("queries.toml"))
    {
        queries = merge_queries(queries, parse_queries_toml(&content));
    }

    // Layer workspace queries
    if let Ok(content) = std::fs::read_to_string(cwd.join(".base").join("queries.toml")) {
        queries = merge_queries(queries, parse_queries_toml(&content));
    }

    // Replace {{prefix}} placeholder in SPARQL text
    for q in &mut queries {
        q.sparql = q.sparql.replace("{{prefix}}", &config.namespace.prefix);
    }

    queries.retain(|q| q.enabled);
    queries.sort_by_key(|q| q.order);
    queries
}

fn parse_queries_toml(content: &str) -> Vec<QueryDef> {
    toml::from_str::<QueriesFile>(content)
        .map(|f| f.query)
        .unwrap_or_default()
}

/// Merge overlay queries onto base: override by name, append new.
fn merge_queries(base: Vec<QueryDef>, overlay: Vec<QueryDef>) -> Vec<QueryDef> {
    let mut merged = base;
    for oq in overlay {
        if let Some(pos) = merged.iter().position(|q| q.name == oq.name) {
            merged[pos] = oq;
        } else {
            merged.push(oq);
        }
    }
    merged
}
