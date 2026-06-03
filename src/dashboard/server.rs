use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::response::Html;
use axum::routing::{delete, get, patch, post, put};
use include_dir::{Dir, include_dir};
use tower_http::cors::CorsLayer;

use crate::config::BaseConfig;

static DASHBOARD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dashboard/dist");

/// Shared state for all API handlers.
pub struct AppState {
    pub store: Mutex<oxigraph::store::Store>,
    pub config: BaseConfig,
    pub cwd: PathBuf,
    pub trig_path: PathBuf,
}

/// Start the dashboard server on the given port.
pub async fn start(port: u16, cwd: PathBuf) {
    let config = BaseConfig::load(&cwd);

    let (store, trig_path) = match crate::crud::load_workspace_store(&cwd) {
        Ok((store, trig_path)) => (store, trig_path),
        Err(e) => {
            eprintln!("Failed to load graph: {e}");
            eprintln!("Run `base scaffold` first, then `base sync` to populate.");
            let base_dir = crate::config::find_workspace_base(&cwd)
                .unwrap_or_else(|| cwd.join(".base"));
            (
                oxigraph::store::Store::new().expect("in-memory store"),
                base_dir.join("graph.trig"),
            )
        }
    };

    // Rotate hook event log if oversized
    super::api::rotate_hook_log(&trig_path);

    let state = Arc::new(AppState {
        store: Mutex::new(store),
        config,
        cwd,
        trig_path,
    });

    let app = Router::new()
        .route("/", get(serve_index))
        // Graph API
        .route("/api/graph/nodes", get(super::api::nodes))
        .route("/api/graph/edges", get(super::api::edges))
        .route("/api/graph/search", get(super::api::search))
        .route("/api/graph/node/{iri}", get(super::api::node_detail))
        .route("/api/graph/node/{iri}/notes", get(super::api::get_notes))
        .route("/api/graph/node/{iri}/notes", post(super::api::add_note))
        .route("/api/graph/node/{iri}/notes/{index}", put(super::api::update_note))
        .route("/api/graph/node/{iri}/notes/{index}", delete(super::api::delete_note))
        // Ops API
        .route("/api/ops/projects", get(super::api::ops_projects))
        .route("/api/ops/decisions", get(super::api::ops_decisions))
        .route("/api/ops/reminders", get(super::api::ops_reminders))
        // Task status
        .route("/api/ops/task/{iri}/status", patch(super::api::update_task_status))
        // Usage Analytics
        .route("/api/usage/summary", get(super::api::usage_summary))
        .route("/api/usage/sessions", get(super::api::usage_sessions))
        .route("/api/usage/projects", get(super::api::usage_projects))
        // Graph management
        .route("/api/graph/reload", post(super::api::reload_graph))
        .route("/api/graph/entity", post(super::api::create_entity))
        // Task creation
        .route("/api/ops/task", post(super::api::create_task))
        // Domain rules
        .route("/api/domains", get(super::api::get_domains))
        .route("/api/domains/rule", post(super::api::add_rule))
        .route("/api/domains/rule", delete(super::api::delete_rule))
        // Export
        .route("/api/export/usage-csv", get(super::api::export_usage_csv))
        .route("/api/export/graph-json", get(super::api::export_graph_json))
        // WebSocket
        .route("/api/ws/session", get(super::api::ws_session))
        .fallback(get(serve_static))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let bind_addr = format!("0.0.0.0:{port}");
    let url = format!("http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind {bind_addr}: {e}");
            std::process::exit(1);
        });

    println!("Dashboard: {url}");
    println!("Press Ctrl+C to stop\n");

    if let Err(e) = open::that(&url) {
        eprintln!("Could not open browser: {e}");
        println!("Open manually: {url}");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");

    println!("\nDashboard stopped.");
}

async fn serve_index() -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    match DASHBOARD_DIR.get_file("index.html") {
        Some(file) => (
            [(header::CONTENT_TYPE, "text/html"), (header::CACHE_CONTROL, "no-cache")],
            file.contents_utf8().unwrap_or(""),
        ).into_response(),
        None => Html("<h1>Dashboard assets not found</h1>".to_string()).into_response(),
    }
}

async fn serve_static(uri: axum::http::Uri) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    let path = uri.path().trim_start_matches('/');

    match DASHBOARD_DIR.get_file(path) {
        Some(file) => {
            let mime = match path.rsplit('.').next() {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("html") => "text/html",
                Some("json") => "application/json",
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                Some("woff2") => "font/woff2",
                Some("woff") => "font/woff",
                _ => "application/octet-stream",
            };
            (StatusCode::OK, [(header::CONTENT_TYPE, mime)], file.contents().to_vec()).into_response()
        }
        None => {
            match DASHBOARD_DIR.get_file("index.html") {
                Some(file) => Html(file.contents_utf8().unwrap_or("").to_string()).into_response(),
                None => (StatusCode::NOT_FOUND, "Not found").into_response(),
            }
        }
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
}
