pub mod active_awareness;
pub mod pulse;
pub mod staleness;
pub mod suppression;

use std::path::Path;

use anyhow::Result;

use crate::config::BaseConfig;

/// A signal result: name, priority (lower = higher), and output text.
struct SignalResult {
    name: String,
    priority: u32,
    output: String,
}

/// Run all signals with suppression and budget cap. Returns combined output.
pub fn run_signals(cwd: &Path, config: &BaseConfig) -> Result<String> {
    if !config.signal.enabled {
        return Ok(String::new());
    }

    let ns = &config.namespace;
    let sig = &config.signal;
    let base_dir = crate::config::find_workspace_base(cwd);

    // Collect signal outputs
    let mut results: Vec<SignalResult> = Vec::new();

    if let Ok(output) = active_awareness::run(cwd, ns, sig)
        && !output.is_empty() {
            results.push(SignalResult { name: "active-awareness".into(), priority: 1, output });
        }
    if let Ok(output) = pulse::run(cwd, ns, sig)
        && !output.is_empty() {
            results.push(SignalResult { name: "pulse".into(), priority: 2, output });
        }
    if let Ok(output) = staleness::run(cwd, ns, sig)
        && !output.is_empty() {
            results.push(SignalResult { name: "staleness".into(), priority: 3, output });
        }

    // Sort by priority
    results.sort_by_key(|r| r.priority);

    // Apply suppression: skip signals whose output hasn't changed
    let mut state = base_dir
        .as_deref()
        .map(suppression::SignalState::load)
        .unwrap_or_default();

    let novel: Vec<&SignalResult> = results
        .iter()
        .filter(|r| {
            let hash = suppression::hash_output(&r.output);
            state.is_novel(&r.name, hash)
        })
        .collect();

    if novel.is_empty() {
        return Ok(String::new());
    }

    // Apply budget cap
    let mut combined = String::new();
    let mut chars_used = 0;
    let mut dropped = 0;

    for result in &novel {
        if result.priority == 1 || chars_used + result.output.len() <= sig.max_chars {
            combined.push_str(&result.output);
            combined.push('\n');
            chars_used += result.output.len();

            // Update suppression state for emitted signals
            let hash = suppression::hash_output(&result.output);
            state.update(&result.name, hash);
        } else {
            dropped += 1;
        }
    }

    if dropped > 0 {
        combined.push_str(&format!("[+{dropped} signals suppressed — budget cap]\n"));
    }

    // Save suppression state
    if let Some(ref base_dir) = base_dir {
        let _ = state.save(base_dir);
    }

    Ok(combined.trim_end().to_string())
}
