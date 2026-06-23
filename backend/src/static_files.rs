use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::path::Path;

use crate::state::SharedState;

pub async fn serve_favicon() -> Response {
    serve_static_file("frontend/dist/favicon.svg", "image/svg+xml").await
}

pub async fn serve_favicon_png() -> Response {
    serve_static_file("frontend/dist/favicon.png", "image/png").await
}

pub async fn serve_service_worker() -> Response {
    serve_static_file("frontend/dist/service-worker.js", "application/javascript").await
}

pub async fn serve_manifest(State(state): State<SharedState>) -> impl IntoResponse {
    let title = &state.site_title;
    let manifest = serde_json::json!({
        "name": title,
        "short_name": title,
        "description": "A stupidly simple todo list",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#ffffff",
        "theme_color": "#000000",
        "icons": [
            {
                "src": "favicon.svg",
                "type": "image/svg+xml",
                "sizes": "any"
            },
            {
                "src": "favicon.png",
                "type": "image/png",
                "sizes": "192x192"
            },
            {
                "src": "favicon.png",
                "type": "image/png",
                "sizes": "512x512"
            }
        ],
        "orientation": "any"
    });
    Json(manifest)
}

fn get_files_recursive(dir: &Path, base: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(get_files_recursive(&path, base));
            } else if let Ok(rel) = path.strip_prefix(base) {
                if let Some(s) = rel.to_str() {
                    let url = format!("/{}", s.replace('\\', "/"));
                    files.push(url);
                }
            }
        }
    }
    files
}

pub fn build_asset_manifest() -> Vec<String> {
    let dist_path = Path::new("frontend/dist");
    let mut files = get_files_recursive(dist_path, dist_path);
    if !files.contains(&"/favicon.svg".to_string()) {
        files.push("/favicon.svg".to_string());
    }
    if !files.contains(&"/manifest.json".to_string()) {
        files.push("/manifest.json".to_string());
    }
    files
}

pub async fn serve_asset_manifest(State(state): State<SharedState>) -> impl IntoResponse {
    Json(state.asset_manifest.clone())
}

async fn serve_static_file(path: &str, content_type: &str) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, content_type)], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
