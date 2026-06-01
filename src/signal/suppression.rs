use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Tracks signal output hashes across sessions for novelty detection.
/// Stored at `.base/.signal-state` (JSON). NOT cleared on session-start —
/// signals persist because novelty is cross-session.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SignalState {
    #[serde(default)]
    pub hashes: HashMap<String, u64>,
}

impl SignalState {
    pub fn load(base_dir: &Path) -> Self {
        let path = base_dir.join(".signal-state");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, base_dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(base_dir)?;
        let path = base_dir.join(".signal-state");
        let json = serde_json::to_string(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn is_novel(&self, name: &str, hash: u64) -> bool {
        self.hashes.get(name) != Some(&hash)
    }

    pub fn update(&mut self, name: &str, hash: u64) {
        self.hashes.insert(name.to_string(), hash);
    }
}

/// Hash signal output text for change detection.
pub fn hash_output(output: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    output.hash(&mut hasher);
    hasher.finish()
}
