//! `todo` backend entrypoint.
//!
//! Configuration is parsed by `shared_backend::server::ServerConfig`,
//! CORS is `shared_backend::middleware::cors_layer`, and security
//! headers + HSTS + title injection come from `shared_backend::middleware`.
//! The remaining middlewares (`auth_middleware`, `rate_limit_middleware`,
//! `origin_validation_middleware`) are todo-specific and live in
//! `middleware.rs`.

use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

pub mod middleware;
mod routes;
mod state;
#[cfg(test)]
mod tests;
mod types;

use middleware::auth::run_todo_migrations;
use middleware::custom::{
    auth_middleware, origin_validation_middleware, rate_limit_middleware,
    security_headers_middleware,
};
use middleware::static_files::{
    build_asset_manifest, serve_asset_manifest, serve_favicon, serve_favicon_png, serve_health,
    serve_index, serve_manifest, serve_service_worker,
};
pub use middleware::{auth, custom, static_files};
use routes::{get_config, get_pin_required, get_todos, logout, save_todos, verify_pin};
use state::AppState;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // ───── tracing setup ─────
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
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
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

    // ───── config ─────
    let server_config = Arc::new(shared_backend::server::ServerConfig::from_env("TODO"));

    let port = server_config.port;
    let allowed_origins = server_config.allowed_origins.clone();
    let is_production =
        std::env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string()) == "production";

    let data_dir = "data";
    let data_file = format!("{data_dir}/todos.json");

    if let Err(e) = std::fs::create_dir_all(data_dir) {
        eprintln!("Failed to create data directory: {e}");
    }
    if !std::path::Path::new(&data_file).exists()
        && let Err(e) = std::fs::write(&data_file, "{}")
    {
        eprintln!("Failed to initialize todos file: {e}");
    }
    run_todo_migrations(&data_file);

    let asset_manifest = build_asset_manifest();

    // ───── build state ─────
    let app_state = Arc::new(AppState {
        pin: server_config.pin.clone(),
        site_title: server_config.site_title.clone(),
        single_list: std::env::var("SINGLE_LIST")
            .map(|v| v == "true")
            .unwrap_or(false),
        allowed_origins: allowed_origins.clone(),
        is_production,
        data_file,
        asset_manifest,
        max_attempts: server_config.max_attempts as usize,
        lockout_duration: server_config.lockout_duration(),
        enable_translation: server_config.enable_translation,
        enable_themes: server_config.enable_themes,
        enable_print: server_config.enable_print,
        show_version: server_config.show_version,
        show_github: server_config.show_github,
        trust_proxy: server_config.trust_proxy,
        trusted_proxies: server_config.trusted_proxies.clone(),
        cookie_max_age_hours: server_config.cookie_max_age_hours,
        active_sessions: RwLock::new(std::collections::HashSet::new()),
        rate_limiter: RwLock::new(HashMap::new()),
    });

    // ───── background cleanup ─────
    // PIN-attempt entries are now stored process-globally by
    // shared_backend::auth::attempts and are cleaned up lazily on read
    // (entries older than lockout_duration are dropped). We only need
    // to clean up the per-IP rate-limit map here.
    let clean_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            clean_state.clean_old_rate_limits().await;
        }
    });

    // ───── middleware layers ─────
    let cors = shared_backend::middleware::cors_layer(&server_config);
    let hsts_state = shared_backend::middleware::HstsState(server_config.clone());
    let title_state = shared_backend::middleware::TitleState(server_config.clone());

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
            rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/favicon.svg", get(serve_favicon))
        .route("/favicon.png", get(serve_favicon_png))
        .route("/manifest.json", get(serve_manifest))
        .route("/asset-manifest.json", get(serve_asset_manifest))
        .route("/service-worker.js", get(serve_service_worker))
        .route("/health", get(serve_health))
        .fallback_service(
            ServeDir::new("frontend/dist").fallback(ServeFile::new("frontend/dist/index.html")),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // security headers from shared-assets (X-Frame-Options, CSP, etc.)
        .layer(axum_middleware::from_fn(security_headers_middleware))
        // HSTS only when HTTPS is in use
        .layer(axum_middleware::from_fn_with_state(
            hsts_state,
            shared_backend::middleware::hsts_layer,
        ))
        // title injection (replaces {{SITE_TITLE}} in HTML)
        .layer(axum_middleware::from_fn_with_state(
            title_state,
            shared_backend::middleware::title_injection_layer,
        ))
        // CORS last so it can short-circuit OPTIONS preflights
        .layer(cors)
        .with_state(app_state.clone());

    // ───── bind & serve ─────
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Todo server running at http://localhost:{port}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
