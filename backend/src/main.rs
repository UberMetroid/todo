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
use handlers::{get_config, get_pin_required, get_todos, logout, save_todos, verify_pin};
use middleware::{auth_middleware, origin_validation_middleware, security_headers_middleware};
use state::AppState;
use static_files::{
    build_asset_manifest, serve_asset_manifest, serve_favicon, serve_favicon_png, serve_manifest,
    serve_service_worker,
};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("LOG_DIR").ok().or_else(|| {
        let data_dir = std::path::Path::new("/app/data");
        if data_dir.is_dir() {
            Some("/app/data/log".to_string())
        } else {
            Some("/app/log".to_string())
        }
    });

    let (file_layer_error, file_layer_app) = if let Some(ref dir) = log_dir {
        if dir == "off" || dir == "none" || dir == "false" {
            (None, None)
        } else {
            let _ = std::fs::create_dir_all(dir);
            let error_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(std::path::Path::new(dir).join("app.log"))
                .ok();

            let error_layer = error_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
            });

            let app_layer = app_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
            });

            (error_layer, app_layer)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer_error)
        .with(file_layer_app)
        .init();

    dotenvy::from_path("/app/data/.env").ok();
    dotenvy::dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "4403".to_string())
        .parse::<u16>()
        .unwrap_or(4403);

    let pin = std::env::var("ADAM_PIN")
        .or_else(|_| std::env::var("PIN"))
        .ok()
        .filter(|p| !p.trim().is_empty());
    let site_title = std::env::var("ADAM_TITLE")
        .or_else(|_| std::env::var("ADAM_SITE_TITLE"))
        .or_else(|_| std::env::var("SITE_TITLE"))
        .unwrap_or_else(|_| "Adam".to_string());
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

    let max_attempts = std::env::var("MAX_ATTEMPTS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5);

    let enable_translation = std::env::var("ENABLE_TRANSLATION")
        .map(|v| v == "true" || v == "on")
        .unwrap_or(false);

    let enable_themes = std::env::var("ENABLE_THEMES")
        .map(|v| v != "false" && v != "off")
        .unwrap_or(true);

    let enable_print = std::env::var("ENABLE_PRINT")
        .map(|v| v != "false" && v != "off")
        .unwrap_or(true);

    let app_state = Arc::new(AppState {
        pin,
        site_title,
        single_list,
        allowed_origins: allowed_origins.clone(),
        is_production,
        data_file,
        asset_manifest,
        max_attempts,
        enable_translation,
        enable_themes,
        enable_print,
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
        .route("/favicon.png", get(serve_favicon_png))
        .route("/manifest.json", get(serve_manifest))
        .route("/asset-manifest.json", get(serve_asset_manifest))
        .route("/service-worker.js", get(serve_service_worker))
        .fallback_service(
            ServeDir::new("frontend/dist").fallback(ServeFile::new("frontend/dist/index.html")),
        )
        .layer(cors)
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Adam server running at http://localhost:{}", port);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
