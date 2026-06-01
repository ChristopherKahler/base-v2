use std::path::Path;

use anyhow::Result;
use oxigraph::sparql::QueryResults;

use crate::config::{NamespaceConfig, SignalConfig};
use crate::crud;

/// Staleness signal: entities not touched within their stale threshold.
/// Priority 3.
pub fn run(cwd: &Path, ns: &NamespaceConfig, sig: &SignalConfig) -> Result<String> {
    let stale_cutoff = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(sig.stale_days as i64))
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
        .unwrap_or_default();

    let p = &ns.prefix;
    let sparql = format!(
        "SELECT ?name ?type ?lastActive WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity a ?type ;\n\
               {p}:name ?name ;\n\
               {p}:lastActive ?lastActive ;\n\
               {p}:status \"active\" .\n\
             FILTER(?type IN ({p}:Project, {p}:App, {p}:Framework, {p}:TrackingProject, {p}:Task))\n\
             FILTER(?lastActive < \"{stale_cutoff}\"^^xsd:dateTime)\n\
           }}\n\
         }}\n\
         ORDER BY ?lastActive\n\
         LIMIT 10"
    );

    let results = crud::load_and_query(cwd, ns, &sparql)?;
    let QueryResults::Solutions(solutions) = results else {
        return Ok(String::new());
    };

    let now = chrono::Local::now();
    let rows: Vec<(String, String)> = solutions
        .filter_map(|r| r.ok())
        .filter_map(|row| {
            let name = row.get("name").map(|t| crud::term_display(t.into()))?;
            let last = row.get("lastActive").map(|t| crud::term_display(t.into()))?;
            let days_ago = chrono::DateTime::parse_from_rfc3339(&last)
                .ok()
                .map(|dt| (now.fixed_offset() - dt).num_days())
                .unwrap_or(0);
            Some((name, format!("{days_ago} days ago")))
        })
        .collect();

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("[Stale Items]\n");
    for (name, age) in &rows {
        output.push_str(&format!("- {name} — last active {age}\n"));
    }

    Ok(output.trim_end().to_string())
}
