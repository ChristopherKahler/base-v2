use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ─── Activation Key ─────────────────────────────────────────
// Replaced before release builds. Distributed via Skool classroom only.
const ACTIVATION_KEY: &str = "PLACEHOLDER_REPLACE_BEFORE_RELEASE";

// ─── Manifest Structs ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub chrisai: ChrisAiSection,
    #[serde(default)]
    pub components: HashMap<String, ComponentEntry>,
    #[serde(default)]
    pub curated: HashMap<String, CuratedEntry>,
    #[serde(default)]
    pub update_check: UpdateCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedEntry {
    pub version: String,
    pub path: String,
    #[serde(default)]
    pub source: String, // e.g. "npm:context-mode"
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChrisAiSection {
    #[serde(default)]
    pub installed_at: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEntry {
    pub version: String,
    pub path: String,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheck {
    #[serde(default)]
    pub last_checked: String,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    #[serde(default)]
    pub pending_update: String,
    #[serde(default)]
    pub dismissed_until: String,
}

fn default_source() -> String {
    "https://chrisai.cv/skool".into()
}

fn default_ttl() -> u64 {
    604800 // 7 days
}

impl Default for ChrisAiSection {
    fn default() -> Self {
        Self {
            installed_at: String::new(),
            source: default_source(),
            token: String::new(),
        }
    }
}

impl Default for UpdateCheck {
    fn default() -> Self {
        Self {
            last_checked: String::new(),
            ttl_seconds: default_ttl(),
            pending_update: String::new(),
            dismissed_until: String::new(),
        }
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            chrisai: ChrisAiSection::default(),
            components: HashMap::new(),
            curated: HashMap::new(),
            update_check: UpdateCheck::default(),
        }
    }
}

// ─── Manifest I/O ───────────────────────────────────────────

impl Manifest {
    /// Resolve path to `~/.base-gbl/manifest.toml`.
    pub fn manifest_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".base-gbl").join("manifest.toml"))
    }

    /// Load manifest from `~/.base-gbl/manifest.toml`. Returns None if missing or unparseable.
    pub fn load() -> Option<Self> {
        let path = Self::manifest_path()?;
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Atomic write to `~/.base-gbl/manifest.toml` (temp + rename).
    pub fn save(&self) -> Result<()> {
        let path = Self::manifest_path().context("Cannot determine home directory")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating {}", parent.display()))?;
        }

        let content =
            toml::to_string_pretty(self).context("Serializing manifest to TOML")?;

        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &content)
            .with_context(|| format!("Writing {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("Renaming {} → {}", tmp.display(), path.display()))?;

        Ok(())
    }

    /// Check if this install is activated (token matches compiled key).
    pub fn is_activated(&self) -> bool {
        !self.chrisai.token.is_empty() && self.chrisai.token == ACTIVATION_KEY
    }
}

// ─── Component Detection ────────────────────────────────────

/// Scan the filesystem for a known component and return its entry if found.
pub fn detect_component(name: &str) -> Option<ComponentEntry> {
    let home = dirs::home_dir()?;
    let now = chrono::Local::now().to_rfc3339();

    match name {
        "base" => {
            let bin = home.join(".local").join("bin").join("base");
            if bin.exists() {
                Some(ComponentEntry {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    path: "~/.local/bin/base".to_string(),
                    installed_at: now,
                })
            } else {
                None
            }
        }
        "paul" => detect_skill_component(&home, "paul-framework", &now),
        "seed" => detect_skill_component(&home, "seed", &now),
        "skillsmith" => detect_skill_component(&home, "skillsmith", &now),
        _ => None,
    }
}

/// Detect a skill component by checking known paths and reading package.json for version.
fn detect_skill_component(home: &Path, name: &str, now: &str) -> Option<ComponentEntry> {
    // Check ~/.claude/paul-framework/ (special case for PAUL) or ~/.claude/commands/{name}/
    let (dir, display_path) = if name == "paul-framework" {
        (
            home.join(".claude").join("paul-framework"),
            "~/.claude/paul-framework".to_string(),
        )
    } else {
        (
            home.join(".claude").join("commands").join(name),
            format!("~/.claude/commands/{name}"),
        )
    };

    if !dir.exists() {
        return None;
    }

    let version = read_package_version(&dir).unwrap_or_else(|| "unknown".to_string());

    Some(ComponentEntry {
        version,
        path: display_path,
        installed_at: now.to_string(),
    })
}

/// Read version from package.json in a directory.
fn read_package_version(dir: &Path) -> Option<String> {
    let pkg = dir.join("package.json");
    let content = std::fs::read_to_string(pkg).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}

// ─── Activation ─────────────────────────────────────────────

/// Validate an activation key and write token to manifest.
pub fn activate(key: &str) -> Result<()> {
    let key = key.trim();

    if key != ACTIVATION_KEY {
        println!("════════════════════════════════════════");
        println!("⛔ Invalid activation key.\n");
        println!("Get your key at https://chrisai.cv/skool");
        println!("════════════════════════════════════════");
        anyhow::bail!("Invalid activation key");
    }

    let mut manifest = Manifest::load().unwrap_or_default();
    manifest.chrisai.token = key.to_string();

    if manifest.chrisai.installed_at.is_empty() {
        manifest.chrisai.installed_at = chrono::Local::now().to_rfc3339();
    }

    manifest.save()?;

    println!("════════════════════════════════════════");
    println!("✓ Activated — attribution removed.\n");
    println!("Thank you for being a ChrisAI member.");
    println!("Chris AI Systems · https://chrisai.cv/skool");
    println!("════════════════════════════════════════");

    Ok(())
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_toml_round_trip() {
        let mut components = HashMap::new();
        components.insert(
            "base".to_string(),
            ComponentEntry {
                version: "0.1.0".to_string(),
                path: "~/.local/bin/base".to_string(),
                installed_at: "2026-06-03T15:00:00-05:00".to_string(),
            },
        );
        components.insert(
            "paul".to_string(),
            ComponentEntry {
                version: "1.4.0".to_string(),
                path: "~/.claude/paul-framework".to_string(),
                installed_at: "2026-06-03T15:00:00-05:00".to_string(),
            },
        );

        let manifest = Manifest {
            chrisai: ChrisAiSection {
                installed_at: "2026-06-03T15:00:00-05:00".to_string(),
                source: "https://chrisai.cv/skool".to_string(),
                token: String::new(),
            },
            components,
            curated: HashMap::new(),
            update_check: UpdateCheck {
                last_checked: "2026-06-03T15:00:00-05:00".to_string(),
                ttl_seconds: 604800,
                pending_update: String::new(),
                dismissed_until: String::new(),
            },
        };

        let serialized = toml::to_string_pretty(&manifest).expect("serialize");
        let deserialized: Manifest = toml::from_str(&serialized).expect("deserialize");

        assert_eq!(deserialized.chrisai.installed_at, manifest.chrisai.installed_at);
        assert_eq!(deserialized.chrisai.source, manifest.chrisai.source);
        assert_eq!(deserialized.chrisai.token, manifest.chrisai.token);
        assert_eq!(deserialized.update_check.ttl_seconds, 604800);
        assert_eq!(deserialized.components.len(), 2);
        assert_eq!(
            deserialized.components["base"].version,
            manifest.components["base"].version
        );
        assert_eq!(
            deserialized.components["paul"].version,
            manifest.components["paul"].version
        );
    }

    #[test]
    fn is_activated_with_valid_token() {
        let manifest = Manifest {
            chrisai: ChrisAiSection {
                token: ACTIVATION_KEY.to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(manifest.is_activated());
    }

    #[test]
    fn is_activated_with_empty_token() {
        let manifest = Manifest::default();
        assert!(!manifest.is_activated());
    }

    #[test]
    fn is_activated_with_wrong_token() {
        let manifest = Manifest {
            chrisai: ChrisAiSection {
                token: "wrong-key".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!manifest.is_activated());
    }
}
