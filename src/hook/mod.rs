pub mod post_tool_use;
pub mod session_start;

use std::io::Read;
use std::path::PathBuf;

use crate::config::BaseConfig;

/// Entry point for all hook events. Fail-open: any error → stderr only, exit 0, empty stdout.
pub fn dispatch(event: &str) {
    if let Err(e) = run(event) {
        eprintln!("base hook {event}: {e:#}");
    }
}

fn run(event: &str) -> anyhow::Result<()> {
    let stdin_json = read_stdin()?;

    let cwd = stdin_json
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let config = BaseConfig::load(&cwd);

    match event {
        "session-start" => session_start::handle(&config, &cwd),
        "post-tool-use" => post_tool_use::handle(&config, &cwd, &stdin_json),
        _ => Ok(()), // Unknown events → silent exit
    }
}

fn read_stdin() -> anyhow::Result<serde_json::Value> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    if buf.trim().is_empty() {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    } else {
        Ok(serde_json::from_str(&buf)?)
    }
}
