use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use super::server::AppState;

// ─── Data types ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct GraphNode {
    pub iri: String,
    pub r#type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub predicate: String,
}

#[derive(Serialize)]
pub struct NodeDetail {
    pub iri: String,
    pub r#type: String,
    pub name: String,
    pub properties: serde_json::Value,
    pub outgoing: Vec<GraphEdge>,
    pub incoming: Vec<GraphEdge>,
    pub notes: Vec<NoteEntry>,
}

#[derive(Serialize, Clone)]
pub struct NoteEntry {
    pub index: u32,
    pub text: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct AddNoteBody {
    pub text: String,
}

#[derive(Serialize)]
pub struct OpsProject {
    pub iri: String,
    pub name: String,
    pub status: String,
    pub path: String,
    pub milestones: Vec<OpsMilestone>,
    pub tasks: Vec<OpsTask>,
}

#[derive(Serialize)]
pub struct OpsMilestone {
    pub iri: String,
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct OpsTask {
    pub iri: String,
    pub name: String,
    pub status: String,
    pub priority: String,
    pub project: String,
    pub milestone: String,
}

#[derive(Serialize)]
pub struct OpsDecision {
    pub name: String,
    pub rationale: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct OpsReminder {
    pub name: String,
    pub due: String,
    pub overdue: bool,
}

#[derive(Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
}

// ─── Graph API ──────────────────────────────────────────────

pub async fn nodes(State(state): State<Arc<AppState>>) -> Json<Vec<GraphNode>> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let store = state.store.lock().unwrap();

    let sparql = format!(
        "{pfx}\n\
         SELECT ?s ?type ?name ?status ?path ?docType WHERE {{\n\
           GRAPH ?g {{\n\
             ?s rdf:type ?type .\n\
             OPTIONAL {{ ?s {p}:name ?name }}\n\
             OPTIONAL {{ ?s {p}:status ?status }}\n\
             OPTIONAL {{ ?s {p}:path ?path }}\n\
             OPTIONAL {{ ?s {p}:documentType ?docType }}\n\
           }}\n\
         }}"
    );

    let mut nodes = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            let iri = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let type_val = row.get("type").map(|t| term_str(t.into())).unwrap_or_default();
            let name = row.get("name").map(|t| term_str(t.into())).unwrap_or_default();
            let status = row.get("status").map(|t| term_str(t.into()));
            let path = row.get("path").map(|t| term_str(t.into()));
            let document_type = row.get("docType").map(|t| term_str(t.into()));
            let tags = fetch_tags(&store, ns, &iri);

            if !iri.is_empty() {
                nodes.push(GraphNode {
                    iri, r#type: short_type(&type_val), name, status, path, document_type, tags,
                });
            }
        }
    }

    Json(nodes)
}

pub async fn edges(State(state): State<Arc<AppState>>) -> Json<Vec<GraphEdge>> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let store = state.store.lock().unwrap();

    let edge_predicates = [
        "relatedTo", "references", "hasRule", "hasDecision",
        "hasMilestone", "hasTask", "calls", "importsFrom",
        "contains", "hasMethod", "belongsTo", "hasTag",
        "hasSection", "operatorNote",
    ];

    let filter = edge_predicates
        .iter()
        .map(|pred| format!("?p = {p}:{pred}"))
        .collect::<Vec<_>>()
        .join(" || ");

    let sparql = format!(
        "{pfx}\nSELECT ?s ?p ?o WHERE {{ GRAPH ?g {{ ?s ?p ?o . FILTER({filter}) }} }}"
    );

    let mut edges = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            let source = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let predicate = row.get("p").map(|t| term_str(t.into())).unwrap_or_default();
            let target = row.get("o").map(|t| term_str(t.into())).unwrap_or_default();

            if !source.is_empty() && !target.is_empty() {
                edges.push(GraphEdge { source, target, predicate: short_predicate(&predicate) });
            }
        }
    }

    Json(edges)
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Json<Vec<GraphNode>> {
    let Some(q) = params.q.filter(|s| !s.is_empty()) else {
        return Json(vec![]);
    };

    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let q_lower = q.to_lowercase();
    let store = state.store.lock().unwrap();

    // Search by name OR by note text (OperatorNote ops:text)
    let sparql = format!(
        "{pfx}\n\
         SELECT ?s ?type ?name ?status WHERE {{\n\
           {{ GRAPH ?g {{\n\
             ?s rdf:type ?type . ?s {p}:name ?name .\n\
             FILTER(CONTAINS(LCASE(?name), \"{q_lower}\"))\n\
           }} }} UNION {{ GRAPH ?g {{\n\
             ?s rdf:type {p}:OperatorNote . ?s {p}:text ?name .\n\
             BIND({p}:OperatorNote AS ?type)\n\
             FILTER(CONTAINS(LCASE(?name), \"{q_lower}\"))\n\
           }} }}\n\
           OPTIONAL {{ GRAPH ?g {{ ?s {p}:status ?status }} }}\n\
         }}"
    );

    let mut nodes = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            let iri = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let type_val = row.get("type").map(|t| term_str(t.into())).unwrap_or_default();
            let name = row.get("name").map(|t| term_str(t.into())).unwrap_or_default();
            let status = row.get("status").map(|t| term_str(t.into()));

            if !iri.is_empty() {
                nodes.push(GraphNode {
                    iri, r#type: short_type(&type_val), name, status,
                    path: None, document_type: None, tags: vec![],
                });
            }
        }
    }

    Json(nodes)
}

pub async fn node_detail(
    State(state): State<Arc<AppState>>,
    Path(iri): Path<String>,
) -> Json<Option<NodeDetail>> {
    let ns = &state.config.namespace;
    let pfx = crate::crud::prefixes(ns);
    let decoded_iri = urldecode(&iri);
    let store = state.store.lock().unwrap();

    let props_sparql = format!(
        "{pfx}\nSELECT ?p ?o WHERE {{ GRAPH ?g {{ <{decoded_iri}> ?p ?o }} }}"
    );

    let mut r#type = String::new();
    let mut name = String::new();
    let mut properties = serde_json::Map::new();
    let mut outgoing = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&props_sparql) {
        for row in solutions.flatten() {
            let pred = row.get("p").map(|t| term_str(t.into())).unwrap_or_default();
            let obj = row.get("o").map(|t| term_str(t.into())).unwrap_or_default();

            if pred.contains("rdf:type") || pred.contains("#type") {
                r#type = short_type(&obj);
            } else if pred.contains(":name") {
                name = obj.clone();
                properties.insert("name".into(), serde_json::Value::String(obj));
            } else {
                let short_pred = short_predicate(&pred);
                if is_edge_predicate(&pred, &ns.prefix) {
                    outgoing.push(GraphEdge {
                        source: decoded_iri.clone(), target: obj, predicate: short_pred,
                    });
                } else {
                    properties.insert(short_pred, serde_json::Value::String(obj));
                }
            }
        }
    }

    if r#type.is_empty() {
        return Json(None);
    }

    let incoming_sparql = format!(
        "{pfx}\nSELECT ?s ?p WHERE {{ GRAPH ?g {{ ?s ?p <{decoded_iri}> }} }}"
    );

    let mut incoming = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&incoming_sparql) {
        for row in solutions.flatten() {
            let source = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let pred = row.get("p").map(|t| term_str(t.into())).unwrap_or_default();
            if !source.is_empty() {
                incoming.push(GraphEdge {
                    source, target: decoded_iri.clone(), predicate: short_predicate(&pred),
                });
            }
        }
    }

    let notes = fetch_notes(&store, ns, &decoded_iri);

    Json(Some(NodeDetail {
        iri: decoded_iri, r#type, name,
        properties: serde_json::Value::Object(properties),
        outgoing, incoming, notes,
    }))
}

// ─── OperatorNotes API ──────────────────────────────────────

pub async fn get_notes(
    State(state): State<Arc<AppState>>,
    Path(iri): Path<String>,
) -> Json<Vec<NoteEntry>> {
    let ns = &state.config.namespace;
    let decoded_iri = urldecode(&iri);
    let store = state.store.lock().unwrap();
    Json(fetch_notes(&store, ns, &decoded_iri))
}

pub async fn add_note(
    State(state): State<Arc<AppState>>,
    Path(iri): Path<String>,
    Json(body): Json<AddNoteBody>,
) -> Result<Json<NoteEntry>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let decoded_iri = urldecode(&iri);

    if body.text.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let store = state.store.lock().unwrap();

    // Get next index
    let existing = fetch_notes(&store, ns, &decoded_iri);
    let next_index = existing.iter().map(|n| n.index).max().unwrap_or(0) + 1;

    let now = crate::crud::now_iso();
    let ws_slug = crate::crud::workspace_slug(&state.cwd);
    let graph = crate::crud::workspace_graph_iri(ns, &ws_slug);

    let note_slug = format!("note-{}-{next_index}",
        decoded_iri.rsplit('/').next().unwrap_or("unknown"));
    let note_iri = crate::crud::build_iri(ns, "note", &note_slug);
    let escaped_text = body.text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

    let sparql = format!(
        "{pfx}\nINSERT DATA {{\n\
           GRAPH <{graph}> {{\n\
             <{note_iri}> rdf:type {p}:OperatorNote ;\n\
               {p}:text \"{escaped_text}\" ;\n\
               {p}:index {next_index} ;\n\
               {p}:createdAt \"{now}\"^^xsd:dateTime .\n\
             <{decoded_iri}> {p}:operatorNote <{note_iri}> .\n\
           }}\n\
         }}"
    );

    if store.update(&sparql).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Persist to disk
    let _ = crate::store::write_back(&store, &state.trig_path);

    Ok(Json(NoteEntry {
        index: next_index,
        text: body.text,
        created_at: now,
    }))
}

pub async fn update_note(
    State(state): State<Arc<AppState>>,
    Path((iri, index)): Path<(String, u32)>,
    Json(body): Json<AddNoteBody>,
) -> Result<Json<NoteEntry>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let decoded_iri = urldecode(&iri);
    let store = state.store.lock().unwrap();

    if body.text.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Find the note IRI by matching parent + index
    let find = format!(
        "{pfx}\nSELECT ?note WHERE {{\n\
           GRAPH ?g {{ <{decoded_iri}> {p}:operatorNote ?note . ?note {p}:index {index} }}\n\
         }}"
    );

    let note_iri = if let Ok(oxigraph::sparql::QueryResults::Solutions(mut sol)) = store.query(&find) {
        sol.find_map(|r| r.ok().and_then(|row| row.get("note").map(|t| term_str(t.into()))))
    } else {
        None
    };

    let Some(note_iri) = note_iri else {
        return Err(StatusCode::NOT_FOUND);
    };

    let ws_slug = crate::crud::workspace_slug(&state.cwd);
    let graph = crate::crud::workspace_graph_iri(ns, &ws_slug);
    let escaped = body.text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

    let sparql = format!(
        "{pfx}\n\
         DELETE {{ GRAPH <{graph}> {{ <{note_iri}> {p}:text ?old }} }}\n\
         INSERT {{ GRAPH <{graph}> {{ <{note_iri}> {p}:text \"{escaped}\" }} }}\n\
         WHERE {{ GRAPH <{graph}> {{ <{note_iri}> {p}:text ?old }} }}"
    );

    if store.update(&sparql).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let _ = crate::store::write_back(&store, &state.trig_path);

    // Get created_at for response
    let created = {
        let q = format!("{pfx}\nSELECT ?c WHERE {{ GRAPH ?g {{ <{note_iri}> {p}:createdAt ?c }} }}");
        if let Ok(oxigraph::sparql::QueryResults::Solutions(mut s)) = store.query(&q) {
            s.find_map(|r| r.ok().and_then(|row| row.get("c").map(|t| term_str(t.into()))))
                .unwrap_or_default()
        } else { String::new() }
    };

    Ok(Json(NoteEntry { index, text: body.text, created_at: created }))
}

pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path((iri, index)): Path<(String, u32)>,
) -> StatusCode {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let decoded_iri = urldecode(&iri);
    let store = state.store.lock().unwrap();

    // Find the note IRI
    let find = format!(
        "{pfx}\nSELECT ?note WHERE {{\n\
           GRAPH ?g {{ <{decoded_iri}> {p}:operatorNote ?note . ?note {p}:index {index} }}\n\
         }}"
    );

    let note_iri = if let Ok(oxigraph::sparql::QueryResults::Solutions(mut sol)) = store.query(&find) {
        sol.find_map(|r| r.ok().and_then(|row| row.get("note").map(|t| term_str(t.into()))))
    } else {
        None
    };

    let Some(note_iri) = note_iri else {
        return StatusCode::NOT_FOUND;
    };

    let ws_slug = crate::crud::workspace_slug(&state.cwd);
    let graph = crate::crud::workspace_graph_iri(ns, &ws_slug);

    // Delete all triples for this note + the edge from parent
    let sparql = format!(
        "{pfx}\n\
         DELETE WHERE {{ GRAPH <{graph}> {{ <{note_iri}> ?p ?o }} }};\n\
         DELETE WHERE {{ GRAPH <{graph}> {{ <{decoded_iri}> {p}:operatorNote <{note_iri}> }} }}"
    );

    if store.update(&sparql).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let _ = crate::store::write_back(&store, &state.trig_path);
    StatusCode::NO_CONTENT
}

fn fetch_notes(store: &oxigraph::store::Store, ns: &crate::config::NamespaceConfig, iri: &str) -> Vec<NoteEntry> {
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let sparql = format!(
        "{pfx}\n\
         SELECT ?text ?index ?created WHERE {{\n\
           GRAPH ?g {{\n\
             <{iri}> {p}:operatorNote ?note .\n\
             ?note {p}:text ?text ;\n\
               {p}:index ?index .\n\
             OPTIONAL {{ ?note {p}:createdAt ?created }}\n\
           }}\n\
         }} ORDER BY ?index"
    );

    let mut notes = Vec::new();
    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            let text = row.get("text").map(|t| term_str(t.into())).unwrap_or_default();
            let index: u32 = row.get("index").map(|t| term_str(t.into())).unwrap_or_default()
                .parse().unwrap_or(0);
            let created_at = row.get("created").map(|t| term_str(t.into())).unwrap_or_default();
            notes.push(NoteEntry { index, text, created_at });
        }
    }
    notes
}

// ─── Ops API ────────────────────────────────────────────────

pub async fn ops_projects(State(state): State<Arc<AppState>>) -> Json<Vec<OpsProject>> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let store = state.store.lock().unwrap();

    // Get projects
    let proj_sparql = format!(
        "{pfx}\nSELECT ?s ?name ?status ?path WHERE {{\n\
           GRAPH ?g {{ ?s rdf:type {p}:Project . ?s {p}:name ?name .\n\
             OPTIONAL {{ ?s {p}:status ?status }}\n\
             OPTIONAL {{ ?s {p}:path ?path }}\n\
           }}\n\
         }}"
    );

    let mut projects = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&proj_sparql) {
        for row in solutions.flatten() {
            let iri = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let name = row.get("name").map(|t| term_str(t.into())).unwrap_or_default();
            let status = row.get("status").map(|t| term_str(t.into())).unwrap_or_else(|| "active".into());
            let path = row.get("path").map(|t| term_str(t.into())).unwrap_or_default();

            // Get milestones for this project
            let ms_sparql = format!(
                "{pfx}\nSELECT ?m ?mname ?mstatus WHERE {{\n\
                   GRAPH ?g {{ <{iri}> {p}:hasMilestone ?m . ?m {p}:name ?mname .\n\
                     OPTIONAL {{ ?m {p}:status ?mstatus }}\n\
                   }}\n\
                 }}"
            );
            let mut milestones = Vec::new();
            if let Ok(oxigraph::sparql::QueryResults::Solutions(ms_sol)) = store.query(&ms_sparql) {
                for mrow in ms_sol.flatten() {
                    milestones.push(OpsMilestone {
                        iri: mrow.get("m").map(|t| term_str(t.into())).unwrap_or_default(),
                        name: mrow.get("mname").map(|t| term_str(t.into())).unwrap_or_default(),
                        status: mrow.get("mstatus").map(|t| term_str(t.into())).unwrap_or_else(|| "active".into()),
                    });
                }
            }

            // Get tasks for this project
            let task_sparql = format!(
                "{pfx}\nSELECT ?t ?tname ?tstatus ?tpri WHERE {{\n\
                   GRAPH ?g {{ <{iri}> {p}:hasTask ?t . ?t {p}:name ?tname .\n\
                     OPTIONAL {{ ?t {p}:status ?tstatus }}\n\
                     OPTIONAL {{ ?t {p}:priority ?tpri }}\n\
                   }}\n\
                 }}"
            );
            let mut tasks = Vec::new();
            if let Ok(oxigraph::sparql::QueryResults::Solutions(t_sol)) = store.query(&task_sparql) {
                for trow in t_sol.flatten() {
                    tasks.push(OpsTask {
                        iri: trow.get("t").map(|t| term_str(t.into())).unwrap_or_default(),
                        name: trow.get("tname").map(|t| term_str(t.into())).unwrap_or_default(),
                        status: trow.get("tstatus").map(|t| term_str(t.into())).unwrap_or_else(|| "active".into()),
                        priority: trow.get("tpri").map(|t| term_str(t.into())).unwrap_or_else(|| "normal".into()),
                        project: name.clone(),
                        milestone: String::new(),
                    });
                }
            }

            projects.push(OpsProject { iri, name, status, path, milestones, tasks });
        }
    }

    Json(projects)
}

pub async fn ops_decisions(State(state): State<Arc<AppState>>) -> Json<Vec<OpsDecision>> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let store = state.store.lock().unwrap();

    let sparql = format!(
        "{pfx}\nSELECT ?name ?rationale ?created WHERE {{\n\
           GRAPH ?g {{ ?d rdf:type {p}:Decision . ?d {p}:name ?name .\n\
             OPTIONAL {{ ?d {p}:rationale ?rationale }}\n\
             OPTIONAL {{ ?d {p}:createdAt ?created }}\n\
           }}\n\
         }} ORDER BY DESC(?created)"
    );

    let mut decisions = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            decisions.push(OpsDecision {
                name: row.get("name").map(|t| term_str(t.into())).unwrap_or_default(),
                rationale: row.get("rationale").map(|t| term_str(t.into())).unwrap_or_default(),
                created_at: row.get("created").map(|t| term_str(t.into())).unwrap_or_default(),
            });
        }
    }

    Json(decisions)
}

pub async fn ops_reminders(State(state): State<Arc<AppState>>) -> Json<Vec<OpsReminder>> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let store = state.store.lock().unwrap();

    let sparql = format!(
        "{pfx}\nSELECT ?name ?due WHERE {{\n\
           GRAPH ?g {{ ?r rdf:type {p}:Reminder . ?r {p}:name ?name .\n\
             OPTIONAL {{ ?r {p}:due ?due }}\n\
           }}\n\
         }} ORDER BY ?due"
    );

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut reminders = Vec::new();

    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            let due = row.get("due").map(|t| term_str(t.into())).unwrap_or_default();
            let overdue = !due.is_empty() && due < today;
            reminders.push(OpsReminder {
                name: row.get("name").map(|t| term_str(t.into())).unwrap_or_default(),
                due, overdue,
            });
        }
    }

    Json(reminders)
}

// ─── Task status update ─────────────────────────────────────

pub async fn update_task_status(
    Path(iri): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateStatusBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let iri = urldecode(&iri);
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);

    let valid = ["active", "in_progress", "blocked", "completed", "pending"];
    if !valid.contains(&body.status.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut store = state.store.lock().unwrap();

    // Verify entity exists
    let check = format!(
        "{pfx}\nASK {{ GRAPH ?g {{ <{iri}> rdf:type ?t }} }}"
    );
    match store.query(&check) {
        Ok(oxigraph::sparql::QueryResults::Boolean(true)) => {}
        _ => return Err(StatusCode::NOT_FOUND),
    }

    // Delete old status, insert new
    let update = format!(
        "{pfx}\n\
         DELETE {{ GRAPH ?g {{ <{iri}> {p}:status ?old }} }}\n\
         INSERT {{ GRAPH ?g {{ <{iri}> {p}:status \"{}\" }} }}\n\
         WHERE  {{ GRAPH ?g {{ <{iri}> {p}:status ?old }} }}",
        body.status
    );
    store.update(&update).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Write back to disk
    crate::store::write_back(&store, &state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "iri": iri,
        "status": body.status,
    })))
}

// ─── Helpers ────────────────────────────────────────────────

fn fetch_tags(store: &oxigraph::store::Store, ns: &crate::config::NamespaceConfig, iri: &str) -> Vec<String> {
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let sparql = format!(
        "{pfx}\nSELECT ?tag WHERE {{ GRAPH ?g {{ <{iri}> {p}:hasTag ?tag }} }}"
    );
    let mut tags = Vec::new();
    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
        for row in solutions.flatten() {
            if let Some(t) = row.get("tag") {
                let val = term_str(t.into());
                if !val.is_empty() { tags.push(val); }
            }
        }
    }
    tags
}

fn term_str(term: oxigraph::model::TermRef<'_>) -> String {
    match term {
        oxigraph::model::TermRef::NamedNode(n) => n.as_str().to_string(),
        oxigraph::model::TermRef::Literal(l) => l.value().to_string(),
        oxigraph::model::TermRef::BlankNode(b) => b.as_str().to_string(),
        _ => String::new(),
    }
}

fn short_type(full: &str) -> String {
    full.rsplit_once('#').or_else(|| full.rsplit_once(':')).or_else(|| full.rsplit_once('/'))
        .map(|(_, short)| short.to_string()).unwrap_or_else(|| full.to_string())
}

fn short_predicate(full: &str) -> String { short_type(full) }

fn is_edge_predicate(pred: &str, prefix: &str) -> bool {
    let edge_preds = [
        "relatedTo", "references", "hasRule", "hasDecision",
        "hasMilestone", "hasTask", "calls", "importsFrom",
        "contains", "hasMethod", "belongsTo", "operatorNote",
    ];
    edge_preds.iter().any(|ep| pred.contains(&format!("{prefix}:{ep}")) || pred.ends_with(&format!("#{ep}")))
}

fn urldecode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().unwrap_or(b'0');
            let lo = chars.next().unwrap_or(b'0');
            let hex = [hi, lo];
            if let Ok(s) = std::str::from_utf8(&hex) {
                if let Ok(val) = u8::from_str_radix(s, 16) {
                    result.push(val as char);
                    continue;
                }
            }
            result.push('%');
            result.push(hi as char);
            result.push(lo as char);
        } else {
            result.push(b as char);
        }
    }
    result
}

// ─── WebSocket: Session Activity ────────────────────────────

/// WebSocket upgrade handler for live session activity feed.
pub async fn ws_session(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_session(socket, state))
}

/// Tail hook-events.jsonl and push new lines to the WebSocket client.
async fn handle_ws_session(mut socket: WebSocket, state: Arc<AppState>) {
    use tokio::time::Duration;

    let log_path = match state.trig_path.parent() {
        Some(base_dir) => base_dir.join("hook-events.jsonl"),
        None => return,
    };

    // Read current file and backfill last 100 lines
    let mut last_line_count = 0;
    if let Ok(content) = std::fs::read_to_string(&log_path) {
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        last_line_count = lines.len();
        let start = lines.len().saturating_sub(100);
        for line in &lines[start..] {
            if socket.send(Message::Text((*line).to_string().into())).await.is_err() {
                return;
            }
        }
    }

    // Tail loop: check file for new lines every 500ms
    loop {
        match tokio::time::timeout(Duration::from_millis(500), socket.recv()).await {
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(_))) | Ok(None) => return,
            Err(_) => {} // Timeout — check the file
        }

        if let Ok(content) = std::fs::read_to_string(&log_path) {
            let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
            if lines.len() > last_line_count {
                for line in &lines[last_line_count..] {
                    if socket.send(Message::Text((*line).to_string().into())).await.is_err() {
                        return;
                    }
                }
                last_line_count = lines.len();
            }
        }
    }
}

// ─── Usage Analytics ────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct UsageSummary {
    pub total_input: u64,
    pub total_output: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub estimated_cost_usd: f64,
    pub session_count: usize,
    pub models: std::collections::HashMap<String, ModelUsage>,
    pub daily: Vec<DailyUsage>,
}

#[derive(Serialize, Clone, Default)]
pub struct ModelUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost: f64,
    pub count: usize,
}

#[derive(Serialize, Clone)]
pub struct DailyUsage {
    pub date: String,
    pub input: u64,
    pub output: u64,
    pub cost: f64,
}

#[derive(Serialize, Clone)]
pub struct SessionUsageEntry {
    pub session_id: String,
    pub project: String,
    pub model: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cost: f64,
    pub started: String,
    pub messages: usize,
}

fn estimate_cost(model: &str, input: u64, output: u64, cache_read: u64, cache_write: u64) -> f64 {
    let (inp_rate, out_rate, cr_rate, cw_rate) = if model.contains("opus") {
        (15.0, 75.0, 1.5, 18.75)
    } else if model.contains("haiku") {
        (0.80, 4.0, 0.08, 1.0)
    } else {
        // sonnet or unknown
        (3.0, 15.0, 0.30, 3.75)
    };
    (input as f64 * inp_rate + output as f64 * out_rate
        + cache_read as f64 * cr_rate + cache_write as f64 * cw_rate)
        / 1_000_000.0
}

fn parse_usage_data(days: u32) -> (UsageSummary, Vec<SessionUsageEntry>) {
    use std::collections::HashMap;

    let claude_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_default();

    let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let mut total_cache_read = 0u64;
    let mut total_cache_write = 0u64;
    let mut total_cost = 0.0f64;
    let mut models: HashMap<String, ModelUsage> = HashMap::new();
    let mut daily_map: HashMap<String, (u64, u64, f64)> = HashMap::new();
    let mut sessions: Vec<SessionUsageEntry> = Vec::new();

    let Ok(projects) = std::fs::read_dir(&claude_dir) else {
        return (UsageSummary {
            total_input: 0, total_output: 0, total_cache_read: 0, total_cache_write: 0,
            estimated_cost_usd: 0.0, session_count: 0, models: HashMap::new(), daily: Vec::new(),
        }, Vec::new());
    };

    for project_dir in projects.flatten() {
        if !project_dir.path().is_dir() { continue; }

        let project_name = project_dir.file_name().to_string_lossy()
            .split('-').last().unwrap_or("unknown").to_string();

        let Ok(files) = std::fs::read_dir(project_dir.path()) else { continue };

        for file in files.flatten() {
            let path = file.path();
            if path.extension().is_none_or(|e| e != "jsonl") { continue; }

            // Skip old files by mtime
            if let Ok(meta) = std::fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    let age = modified.elapsed().unwrap_or_default();
                    if age > std::time::Duration::from_secs(days as u64 * 24 * 3600) { continue; }
                }
            }

            let session_id = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let Ok(content) = std::fs::read_to_string(&path) else { continue };

            let mut sess_input = 0u64;
            let mut sess_output = 0u64;
            let mut sess_cache_read = 0u64;
            let mut sess_cache_write = 0u64;
            let mut sess_model = String::new();
            let mut sess_started = String::new();
            let mut sess_msgs = 0usize;

            for line in content.lines() {
                let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else { continue };

                let Some(msg) = val.get("message") else { continue };
                let Some(usage) = msg.get("usage") else { continue };

                let model = msg.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
                let inp = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let out = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cr = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cw = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

                let cost = estimate_cost(model, inp, out, cr, cw);

                total_input += inp;
                total_output += out;
                total_cache_read += cr;
                total_cache_write += cw;
                total_cost += cost;

                sess_input += inp;
                sess_output += out;
                sess_cache_read += cr;
                sess_cache_write += cw;
                sess_msgs += 1;
                if sess_model.is_empty() { sess_model = model.to_string(); }

                // Extract timestamp for daily aggregation
                if let Some(ts) = val.get("timestamp").and_then(|t| t.as_str()) {
                    let date = &ts[..10.min(ts.len())];
                    if date >= cutoff_str.as_str() {
                        let entry = daily_map.entry(date.to_string()).or_insert((0, 0, 0.0));
                        entry.0 += inp;
                        entry.1 += out;
                        entry.2 += cost;
                    }
                    if sess_started.is_empty() { sess_started = ts.to_string(); }
                }

                let m = models.entry(model.to_string()).or_default();
                m.input += inp;
                m.output += out;
                m.cache_read += cr;
                m.cache_write += cw;
                m.cost += cost;
                m.count += 1;
            }

            if sess_msgs > 0 {
                sessions.push(SessionUsageEntry {
                    session_id,
                    project: project_name.clone(),
                    model: sess_model,
                    input: sess_input,
                    output: sess_output,
                    cache_read: sess_cache_read,
                    cost: estimate_cost("", sess_input, sess_output, sess_cache_read, sess_cache_write),
                    started: sess_started,
                    messages: sess_msgs,
                });
            }
        }
    }

    // Sort daily by date
    let mut daily: Vec<DailyUsage> = daily_map.into_iter()
        .map(|(date, (input, output, cost))| DailyUsage { date, input, output, cost })
        .collect();
    daily.sort_by(|a, b| a.date.cmp(&b.date));

    // Sort sessions newest-first (no cap — show all within the time window)
    sessions.sort_by(|a, b| b.started.cmp(&a.started));

    let summary = UsageSummary {
        total_input,
        total_output,
        total_cache_read,
        total_cache_write,
        estimated_cost_usd: (total_cost * 100.0).round() / 100.0,
        session_count: sessions.len(),
        models,
        daily,
    };

    (summary, sessions)
}

#[derive(Deserialize)]
pub struct UsageQuery {
    pub days: Option<u32>,
}

pub async fn usage_summary(Query(q): Query<UsageQuery>) -> Json<UsageSummary> {
    let (summary, _) = parse_usage_data(q.days.unwrap_or(30));
    Json(summary)
}

pub async fn usage_sessions(Query(q): Query<UsageQuery>) -> Json<Vec<SessionUsageEntry>> {
    let (_, sessions) = parse_usage_data(q.days.unwrap_or(30));
    Json(sessions)
}

// ─── Graph Reload ───────────────────────────────────────────

pub async fn reload_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let new_store = crate::store::load_graph(&state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut store = state.store.lock().unwrap();
    *store = new_store;

    Ok(Json(serde_json::json!({ "reloaded": true })))
}

// ─── Task Creation ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateTaskBody {
    pub name: String,
    pub project: String,
    pub status: Option<String>,
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTaskBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let status = body.status.as_deref().unwrap_or("active");
    let slug = crate::crud::slugify(&body.name);
    let task_iri = crate::crud::build_iri(ns, "task", &slug);
    let project_slug = crate::crud::slugify(&body.project);
    let project_iri = crate::crud::build_iri(ns, "project", &project_slug);

    let mut store = state.store.lock().unwrap();

    let insert = format!(
        "{pfx}\nINSERT DATA {{\n  GRAPH <{graph}> {{\n\
           <{task_iri}> rdf:type {p}:Task .\n\
           <{task_iri}> {p}:name \"{name}\" .\n\
           <{task_iri}> {p}:status \"{status}\" .\n\
           <{project_iri}> {p}:hasTask <{task_iri}> .\n\
           <{task_iri}> {p}:belongsTo <{project_iri}> .\n\
         }}\n}}",
        graph = crate::crud::workspace_graph_iri(ns, &crate::crud::workspace_slug(&state.cwd)),
        name = body.name.replace('"', "\\\""),
    );

    store.update(&insert).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::store::write_back(&store, &state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "iri": task_iri, "name": body.name, "status": status })))
}

// ─── Entity Creation ────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateEntityBody {
    pub name: String,
    pub r#type: String,
    pub domain: Option<String>,
}

pub async fn create_entity(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateEntityBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let slug = crate::crud::slugify(&body.name);
    let entity_iri = crate::crud::build_iri(ns, "entity", &slug);
    let ws_slug = crate::crud::workspace_slug(&state.cwd);
    let graph = crate::crud::workspace_graph_iri(ns, &ws_slug);

    let mut domain_triple = String::new();
    if let Some(ref domain) = body.domain {
        let domain_slug = crate::crud::slugify(domain);
        let domain_iri = crate::crud::build_iri(ns, "domain", &domain_slug);
        domain_triple = format!("<{entity_iri}> {p}:hasDomain <{domain_iri}> .\n");
    }

    let mut store = state.store.lock().unwrap();

    let insert = format!(
        "{pfx}\nINSERT DATA {{\n  GRAPH <{graph}> {{\n\
           <{entity_iri}> rdf:type {p}:{etype} .\n\
           <{entity_iri}> {p}:name \"{name}\" .\n\
           {domain_triple}\
         }}\n}}",
        etype = body.r#type,
        name = body.name.replace('"', "\\\""),
    );

    store.update(&insert).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::store::write_back(&store, &state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "iri": entity_iri, "name": body.name, "type": body.r#type })))
}

// ─── Domain Rules ───────────────────────────────────────────

#[derive(Serialize)]
pub struct DomainInfo {
    pub name: String,
    pub mode: String,
    pub prompt_keywords: Vec<String>,
    pub paths: Vec<String>,
    pub rules: Vec<String>,
    pub sticky: bool,
}

pub async fn get_domains(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<DomainInfo>> {
    let domains = crate::domain::load_domains(&state.cwd);
    let store = state.store.lock().unwrap();
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);

    let result: Vec<DomainInfo> = domains.iter().map(|d| {
        // Merge TOML rules with graph-native rules
        let mut rules = d.rules.clone();

        let domain_slug = crate::crud::slugify(&d.name);
        let domain_iri = crate::crud::build_iri(ns, "domain", &domain_slug);
        let sparql = format!(
            "{pfx}\nSELECT ?text WHERE {{\n\
               GRAPH ?g {{ <{domain_iri}> {p}:hasRule ?rule . ?rule {p}:ruleText ?text }}\n\
             }} ORDER BY ?text"
        );

        if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&sparql) {
            for row in solutions.flatten() {
                if let Some(t) = row.get("text") {
                    let text = term_str(t.into());
                    if !text.is_empty() && !rules.contains(&text) {
                        rules.push(text);
                    }
                }
            }
        }

        DomainInfo {
            name: d.name.clone(),
            mode: if d.is_always() { "always".into() } else { "triggered".into() },
            prompt_keywords: d.prompt_keywords.clone(),
            paths: d.paths.clone(),
            rules,
            sticky: d.sticky,
        }
    }).collect();
    Json(result)
}

// ─── Rule CRUD ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AddRuleBody {
    pub domain: String,
    pub text: String,
}

pub async fn add_rule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddRuleBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let domain_slug = crate::crud::slugify(&body.domain);
    let domain_iri = crate::crud::build_iri(ns, "domain", &domain_slug);
    let rule_slug = crate::crud::slugify(&body.text);
    let rule_iri = crate::crud::build_iri(ns, "rule", &rule_slug);
    let ws_slug = crate::crud::workspace_slug(&state.cwd);
    let graph = crate::crud::workspace_graph_iri(ns, &ws_slug);

    let mut store = state.store.lock().unwrap();

    let insert = format!(
        "{pfx}\nINSERT DATA {{\n  GRAPH <{graph}> {{\n\
           <{rule_iri}> rdf:type {p}:Rule .\n\
           <{rule_iri}> {p}:ruleText \"{}\" .\n\
           <{domain_iri}> {p}:hasRule <{rule_iri}> .\n\
         }}\n}}",
        body.text.replace('"', "\\\""),
    );

    store.update(&insert).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::store::write_back(&store, &state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "rule_iri": rule_iri, "domain": body.domain, "text": body.text })))
}

#[derive(Deserialize)]
pub struct DeleteRuleBody {
    pub domain: String,
    pub text: String,
}

pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteRuleBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ns = &state.config.namespace;
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    let rule_slug = crate::crud::slugify(&body.text);
    let rule_iri = crate::crud::build_iri(ns, "rule", &rule_slug);

    let mut store = state.store.lock().unwrap();

    let delete = format!(
        "{pfx}\nDELETE WHERE {{ GRAPH ?g {{ <{rule_iri}> ?p ?o }} }};\n\
         DELETE WHERE {{ GRAPH ?g {{ ?s {p}:hasRule <{rule_iri}> }} }}"
    );

    store.update(&delete).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::store::write_back(&store, &state.trig_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── Export ─────────────────────────────────────────────────

pub async fn export_usage_csv(Query(q): Query<UsageQuery>) -> axum::response::Response {
    use axum::http::header;

    let (_, sessions) = parse_usage_data(q.days.unwrap_or(30));
    let mut csv = String::from("session_id,project,model,input_tokens,output_tokens,cost_usd,started,messages\n");
    for s in &sessions {
        csv.push_str(&format!(
            "{},{},{},{},{},{:.4},{},{}\n",
            s.session_id, s.project, s.model, s.input, s.output, s.cost, s.started, s.messages
        ));
    }

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"usage.csv\""),
        ],
        csv,
    ).into_response()
}

pub async fn export_graph_json(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let ns = &state.config.namespace;
    let store = state.store.lock().unwrap();

    // Reuse existing nodes + edges queries
    let nodes_sparql = build_nodes_sparql(ns);
    let edges_sparql = build_edges_sparql(ns);

    let mut nodes_list = Vec::new();
    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&nodes_sparql) {
        for row in solutions.flatten() {
            let name = row.get("name").map(|t| term_str(t.into())).unwrap_or_default();
            let iri = row.get("entity").map(|t| term_str(t.into())).unwrap_or_default();
            nodes_list.push(serde_json::json!({"iri": iri, "name": name}));
        }
    }

    let mut edges_list = Vec::new();
    if let Ok(oxigraph::sparql::QueryResults::Solutions(solutions)) = store.query(&edges_sparql) {
        for row in solutions.flatten() {
            let s = row.get("s").map(|t| term_str(t.into())).unwrap_or_default();
            let p_val = row.get("p").map(|t| term_str(t.into())).unwrap_or_default();
            let o = row.get("o").map(|t| term_str(t.into())).unwrap_or_default();
            edges_list.push(serde_json::json!({"source": s, "predicate": p_val, "target": o}));
        }
    }

    Json(serde_json::json!({
        "nodes": nodes_list,
        "edges": edges_list,
        "exported_at": chrono::Local::now().to_rfc3339(),
    }))
}

/// Build nodes SPARQL (extracted for reuse by export)
fn build_nodes_sparql(ns: &crate::config::NamespaceConfig) -> String {
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    format!(
        "{pfx}\nSELECT DISTINCT ?entity ?type ?name ?status ?path WHERE {{\n\
           GRAPH ?g {{\n\
             ?entity rdf:type ?type .\n\
             OPTIONAL {{ ?entity {p}:name ?name }}\n\
             OPTIONAL {{ ?entity {p}:status ?status }}\n\
             OPTIONAL {{ ?entity {p}:path ?path }}\n\
           }}\n\
         }}"
    )
}

/// Build edges SPARQL (extracted for reuse by export)
fn build_edges_sparql(ns: &crate::config::NamespaceConfig) -> String {
    let p = &ns.prefix;
    let pfx = crate::crud::prefixes(ns);
    format!(
        "{pfx}\nSELECT ?s ?p ?o WHERE {{\n\
           GRAPH ?g {{ ?s ?p ?o }}\n\
           FILTER(isIRI(?o))\n\
           FILTER(?p != rdf:type)\n\
         }}"
    )
}

// ─── Log Rotation ───────────────────────────────────────────

/// Truncate hook-events.jsonl if > 10MB, keeping last 5000 lines.
pub fn rotate_hook_log(trig_path: &std::path::Path) {
    let Some(base_dir) = trig_path.parent() else { return };
    let log_path = base_dir.join("hook-events.jsonl");

    let Ok(meta) = std::fs::metadata(&log_path) else { return };
    if meta.len() < 10 * 1024 * 1024 { return; }

    let Ok(content) = std::fs::read_to_string(&log_path) else { return };
    let lines: Vec<&str> = content.lines().collect();
    let keep = lines.len().saturating_sub(5000);
    let tail: String = lines[keep..].join("\n") + "\n";
    let _ = std::fs::write(&log_path, tail);
}
