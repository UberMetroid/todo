use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

mod auth;
mod handlers;
mod middleware;
mod state;
mod static_files;
#[cfg(test)]
mod tests;

use auth::run_todo_migrations;
use handlers::{get_config, get_pin_required, get_todos, save_todos, verify_pin, logout};
use middleware::{auth_middleware, origin_validation_middleware};
use state::AppState;
use static_files::{
    build_asset_manifest, serve_asset_manifest, serve_favicon, serve_manifest, serve_service_worker,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "4403".to_string())
        .parse::<u16>()
        .unwrap_or(4403);

    let pin = std::env::var("RUSTDO_PIN")
        .ok()
        .filter(|p| !p.trim().is_empty());
    let site_title = std::env::var("RUSTDO_TITLE")
        .or_else(|_| std::env::var("RUSTDO_SITE_TITLE"))
        .or_else(|_| std::env::var("SITE_TITLE"))
        .unwrap_or_else(|_| "RustDo".to_string());
    let single_list = std::env::var("SINGLE_LIST")
        .map(|val| val == "true")
        .unwrap_or(false);
    let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    let node_env = std::env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string());
    let is_production = node_env == "production";

    let data_dir = "data";
    let data_file = format!("{}/todos.json", data_dir);

    if let Err(e) = std::fs::create_dir_all(data_dir) {
        eprintln!("Failed to create data directory: {}", e);
    }

    if !std::path::Path::new(&data_file).exists() {
        if let Err(e) = std::fs::write(&data_file, "{}") {
            eprintln!("Failed to initialize todos file: {}", e);
        }
    }

    run_todo_migrations(&data_file);

    let asset_manifest = build_asset_manifest();

    let app_state = Arc::new(AppState {
        pin,
        site_title,
        single_list,
        allowed_origins: allowed_origins.clone(),
        is_production,
        data_file,
        asset_manifest,
        login_attempts: RwLock::new(HashMap::new()),
    });

    let clean_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let mut attempts = clean_state.login_attempts.write().await;
            attempts.retain(|_, (_, last_time)| last_time.elapsed() < Duration::from_secs(15 * 60));
        }
    });

    let cors = CorsLayer::new()
        .allow_methods(vec![axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderName::from_static("x-pin"),
        ])
        .allow_credentials(true);

    let cors = if allowed_origins == "*" {
        cors.allow_origin(tower_http::cors::AllowOrigin::mirror_request())
    } else {
        let mut origins = Vec::new();
        for origin in allowed_origins.split(',') {
            if let Ok(parsed) = origin.trim().parse() {
                origins.push(parsed);
            }
        }
        cors.allow_origin(origins)
    };

    let protected_routes = Router::new()
        .route("/todos", get(get_todos).post(save_todos))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    let api_routes = Router::new()
        .route("/pin-required", get(get_pin_required))
        .route("/verify-pin", post(verify_pin))
        .route("/config", get(get_config))
        .route("/logout", post(logout))
        .merge(protected_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/favicon.svg", get(serve_favicon))
        .route("/manifest.json", get(serve_manifest))
        .route("/asset-manifest.json", get(serve_asset_manifest))
        .route("/service-worker.js", get(serve_service_worker))
        .fallback_service(
            ServeDir::new("frontend/dist").fallback(ServeFile::new("frontend/dist/index.html")),
        )
        .layer(cors)
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("RustDo server running at http://localhost:{}", port);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
