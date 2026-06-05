use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::{FlowConfig, NamespaceConfig};
use crate::crud;

/// Flow resurface signal: surfaces items that need attention.
/// Three sub-queries: blocked-by scan, stale detection, deferred orphan scan.
/// Priority 2 (competes with pulse for budget space).
pub fn run(cwd: &Path, ns: &NamespaceConfig, flow: &FlowConfig) -> Result<String> {
    let mut sections: Vec<String> = Vec::new();

    // Sub-query 1: Blocked-by scan
    // Find entities marked "blocked" whose blocker is now "completed" or "active"
    if let Ok(output) = blocked_by_scan(cwd, ns) {
        if !output.is_empty() {
            sections.push(output);
        }
    }

    // Sub-query 2: Stale active detection
    // Find active entities with no activity beyond threshold
    if let Ok(output) = stale_detection(cwd, ns, flow.stale_threshold_days) {
        if !output.is_empty() {
            sections.push(output);
        }
    }

    // Sub-query 3: Deferred orphan scan
    // Find deferred entities with expired resurface dates
    if let Ok(output) = deferred_orphan_scan(cwd, ns) {
        if !output.is_empty() {
            sections.push(output);
        }
    }

    if sections.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("<flow-resurface>\n");
    output.push_str(&sections.join("\n"));
    output.push_str("\n</flow-resurface>");

    Ok(output)
}

/// Find entities with status "blocked" whose blocker entity has status "completed" or "active".
/// These items just unblocked and need attention.
fn blocked_by_scan(cwd: &Path, ns: &NamespaceConfig) -> Result<String> {
    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?blockerName ?blockerStatus WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity {p}:name ?name ;\n\
               {p}:status \"blocked\" ;\n\
               {p}:blockedBy ?blocker .\n\
             ?blocker {p}:name ?blockerName ;\n\
               {p}:status ?blockerStatus .\n\
             FILTER(?blockerStatus IN (\"completed\", \"active\"))\n\
           }}\n\
         }}\n\
         ORDER BY ?name"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    let QueryResults::Solutions(solutions) = results else {
        return Ok(String::new());
    };

    let rows: Vec<(String, String)> = solutions
        .filter_map(|r| r.ok())
        .map(|row| {
            (
                row.get("name").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                row.get("blockerName").map(|t| crud::term_display(t.into())).unwrap_or_default(),
            )
        })
        .collect();

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("[Unblocked]\n");
    for (name, blocker) in &rows {
        output.push_str(&format!("- {name} (was blocked by {blocker})\n"));
    }

    Ok(output)
}

/// Find active projects/tasks with lastActive older than the stale threshold.
fn stale_detection(cwd: &Path, ns: &NamespaceConfig, stale_threshold_days: u32) -> Result<String> {
    let cutoff = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(stale_threshold_days as i64))
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
        .unwrap_or_default();

    let now = chrono::Local::now();

    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?type ?lastActive WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity a ?type ;\n\
               {p}:name ?name ;\n\
               {p}:status \"active\" ;\n\
               {p}:lastActive ?lastActive .\n\
             FILTER(?type IN ({p}:Project, {p}:Task, {p}:App, {p}:Framework))\n\
             FILTER(?lastActive < \"{cutoff}\"^^xsd:dateTime)\n\
           }}\n\
         }}\n\
         ORDER BY ?lastActive"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    let QueryResults::Solutions(solutions) = results else {
        return Ok(String::new());
    };

    let rows: Vec<(String, String, String)> = solutions
        .filter_map(|r| r.ok())
        .map(|row| {
            (
                row.get("name").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                row.get("type").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                row.get("lastActive").map(|t| crud::term_display(t.into())).unwrap_or_default(),
            )
        })
        .collect();

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("[Stale]\n");
    for (name, entity_type, last_active) in &rows {
        let days_ago = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_active) {
            let diff = now.signed_duration_since(dt);
            format!("{} days ago", diff.num_days())
        } else {
            "unknown".into()
        };
        output.push_str(&format!("- {name} ({entity_type}, last active {days_ago})\n"));
    }

    Ok(output)
}

/// Find deferred entities with a resurfaceAt date in the past.
fn deferred_orphan_scan(cwd: &Path, ns: &NamespaceConfig) -> Result<String> {
    let now_str = chrono::Local::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false);

    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?resurfaceAt WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity {p}:name ?name ;\n\
               {p}:status \"deferred\" ;\n\
               {p}:resurfaceAt ?resurfaceAt .\n\
             FILTER(?resurfaceAt < \"{now_str}\"^^xsd:dateTime)\n\
           }}\n\
         }}\n\
         ORDER BY ?resurfaceAt"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    let QueryResults::Solutions(solutions) = results else {
        return Ok(String::new());
    };

    let rows: Vec<(String, String)> = solutions
        .filter_map(|r| r.ok())
        .map(|row| {
            (
                row.get("name").map(|t| crud::term_display(t.into())).unwrap_or_default(),
                row.get("resurfaceAt").map(|t| crud::term_display(t.into())).unwrap_or_default(),
            )
        })
        .collect();

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("[Resurface]\n");
    for (name, resurface_at) in &rows {
        output.push_str(&format!("- {name} (deferred until {resurface_at}, now past due)\n"));
    }

    Ok(output)
}
