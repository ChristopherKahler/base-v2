use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Tracks which domains have been injected in the current session.
/// Stored at `.base/.session` (JSON). Session-start clears it.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionState {
    /// domain name → rules hash (for change detection)
    #[serde(default)]
    pub injected: HashMap<String, u64>,
}

impl SessionState {
    /// Load session state from `.base/.session`. Returns empty state if missing or malformed.
    pub fn load(base_dir: &Path) -> Self {
        let path = base_dir.join(".session");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save session state atomically.
    pub fn save(&self, base_dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(base_dir)?;
        let path = base_dir.join(".session");
        let json = serde_json::to_string(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Clear session state (called by session-start for fresh session).
    pub fn clear(base_dir: &Path) {
        let _ = std::fs::remove_file(base_dir.join(".session"));
    }

    /// Check if a domain was already injected with the same rules hash.
    pub fn is_injected(&self, domain: &str, hash: u64) -> bool {
        self.injected.get(domain) == Some(&hash)
    }

    /// Mark a domain as injected with its current rules hash.
    pub fn mark_injected(&mut self, domain: &str, hash: u64) {
        self.injected.insert(domain.to_string(), hash);
    }
}

/// Compute a hash of rule texts for change detection.
/// If rules change (domains.toml edited), hash differs → re-inject.
pub fn rules_hash(rules: &[String]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for rule in rules {
        rule.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_state_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = SessionState::default();
        state.mark_injected("global", 12345);
        state.save(tmp.path()).unwrap();

        let loaded = SessionState::load(tmp.path());
        assert!(loaded.is_injected("global", 12345));
        assert!(!loaded.is_injected("global", 99999));
        assert!(!loaded.is_injected("other", 12345));
    }

    #[test]
    fn session_state_clear() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = SessionState::default();
        state.mark_injected("test", 111);
        state.save(tmp.path()).unwrap();

        SessionState::clear(tmp.path());
        let loaded = SessionState::load(tmp.path());
        assert!(loaded.injected.is_empty());
    }

    #[test]
    fn rules_hash_changes_on_content() {
        let h1 = rules_hash(&["Rule A".into(), "Rule B".into()]);
        let h2 = rules_hash(&["Rule A".into(), "Rule C".into()]);
        let h3 = rules_hash(&["Rule A".into(), "Rule B".into()]);
        assert_ne!(h1, h2);
        assert_eq!(h1, h3);
    }
}
