use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::{NamespaceConfig, SignalConfig};
use crate::crud;

/// Pulse signal: workspace health summary — counts of active/blocked/completed items.
/// Priority 2.
pub fn run(cwd: &Path, ns: &NamespaceConfig, _sig: &SignalConfig) -> Result<String> {
    let p = &ns.prefix;

    // Count projects by status
    let project_sparql = format!(
        "SELECT ?status (COUNT(?proj) AS ?count) WHERE {{\n\
           GRAPH ?g {{ ?proj a ?type ; {p}:status ?status .\n\
             FILTER(?type IN ({p}:Project, {p}:App, {p}:Framework, {p}:TrackingProject))\n\
           }}\n\
         }} GROUP BY ?status"
    );

    let mut active = 0u32;
    let mut blocked = 0u32;
    let mut completed = 0u32;

    if let Ok(QueryResults::Solutions(solutions)) = crud::load_and_query(cwd, ns, &project_sparql) {
        for row in solutions.flatten() {
            let status = row.get("status").map(|t| crud::term_display(t.into())).unwrap_or_default();
            let count: u32 = row
                .get("count")
                .map(|t| crud::term_display(t.into()))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            match status.as_str() {
                "active" => active = count,
                "blocked" => blocked = count,
                "completed" | "done" => completed = count,
                _ => {}
            }
        }
    }

    // Count open tasks
    let task_sparql = format!(
        "SELECT (COUNT(?t) AS ?count) WHERE {{\n\
           GRAPH ?g {{ ?t a {p}:Task ; {p}:status \"active\" }}\n\
         }}"
    );
    let open_tasks = count_query(cwd, ns, &task_sparql);

    // Count overdue reminders
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let reminder_sparql = format!(
        "SELECT (COUNT(?r) AS ?count) WHERE {{\n\
           GRAPH ?g {{ ?r a {p}:Reminder ; {p}:dueDate ?due . FILTER(?due < \"{today}\"^^xsd:date) }}\n\
         }}"
    );
    let overdue = count_query(cwd, ns, &reminder_sparql);

    // Count recent decisions (last 7 days)
    let cutoff = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(7))
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
        .unwrap_or_default();
    let decision_sparql = format!(
        "SELECT (COUNT(?d) AS ?count) WHERE {{\n\
           GRAPH ?g {{ ?d a {p}:Decision ; {p}:createdAt ?ts . FILTER(?ts > \"{cutoff}\"^^xsd:dateTime) }}\n\
         }}"
    );
    let recent_decisions = count_query(cwd, ns, &decision_sparql);

    // Only emit if there's data
    if active + blocked + completed + open_tasks + overdue + recent_decisions == 0 {
        return Ok(String::new());
    }

    let mut output = String::from("[Workspace Pulse]\n");
    output.push_str(&format!("Projects: {active} active, {blocked} blocked, {completed} completed\n"));
    if open_tasks > 0 {
        output.push_str(&format!("Tasks: {open_tasks} open\n"));
    }
    if overdue > 0 {
        output.push_str(&format!("Reminders: {overdue} overdue\n"));
    }
    if recent_decisions > 0 {
        output.push_str(&format!("Decisions: {recent_decisions} this week\n"));
    }

    Ok(output.trim_end().to_string())
}

fn count_query(cwd: &Path, ns: &NamespaceConfig, sparql: &str) -> u32 {
    crud::load_and_query(cwd, ns, sparql)
        .ok()
        .and_then(|r| {
            if let QueryResults::Solutions(solutions) = r {
                solutions.flatten().next().and_then(|row| {
                    row.get("count")
                        .map(|t| crud::term_display(t.into()))
                        .and_then(|s| s.parse().ok())
                })
            } else {
                None
            }
        })
        .unwrap_or(0)
}
