pub mod post_tool_use;
pub mod pre_tool_use;
pub mod session_start;
pub mod user_prompt_submit;

use std::io::Read;
use std::path::PathBuf;

use crate::config::BaseConfig;

/// Data captured from hook execution for event logging.
#[derive(Debug, Default, serde::Serialize)]
pub struct HookEventData {
    pub domains_matched: Vec<String>,
    pub rules_injected: usize,
    pub suppressed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_num: Option<u32>,
}

/// Entry point for all hook events. Fail-open: any error → stderr only, exit 0, empty stdout.
pub fn dispatch(event: &str) {
    let result = run(event);
    let (success, data) = match &result {
        Ok(d) => (true, Some(d)),
        Err(_) => (false, None),
    };
    log_hook_event(event, success, data);
    if let Err(e) = result {
        eprintln!("base hook {event}: {e:#}");
    }
}

fn run(event: &str) -> anyhow::Result<HookEventData> {
    let stdin_json = read_stdin()?;

    let cwd = stdin_json
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let config = BaseConfig::load(&cwd);

    match event {
        "session-start" => {
            session_start::handle(&config, &cwd)?;
            Ok(HookEventData::default())
        }
        "pre-tool-use" => {
            pre_tool_use::handle(&config, &cwd, &stdin_json)?;
            Ok(HookEventData::default())
        }
        "post-tool-use" => {
            post_tool_use::handle(&config, &cwd, &stdin_json)?;
            Ok(HookEventData::default())
        }
        "user-prompt-submit" => user_prompt_submit::handle(&config, &cwd, &stdin_json),
        _ => Ok(HookEventData::default()),
    }
}

/// Append a hook event to the JSONL log file. Fire-and-forget — never blocks hooks.
fn log_hook_event(hook: &str, success: bool, data: Option<&HookEventData>) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let base_dir = match crate::config::find_workspace_base(&cwd) {
        Some(d) => d,
        None => return,
    };

    let log_path = base_dir.join("hook-events.jsonl");
    let ts = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let empty = Vec::new();

    let event = serde_json::json!({
        "ts": ts,
        "hook": hook,
        "success": success,
        "domains_matched": data.map(|d| &d.domains_matched).unwrap_or(&empty),
        "rules_injected": data.map(|d| d.rules_injected).unwrap_or(0),
        "suppressed": data.map(|d| d.suppressed).unwrap_or(0),
        "prompt_num": data.and_then(|d| d.prompt_num),
    });

    use std::io::Write;
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut f| writeln!(f, "{}", event));
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
