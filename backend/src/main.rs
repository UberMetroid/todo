use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use rand::Rng;
use shared::{
    PinRequiredResponse, SiteConfig, TodoLists, VerifyPinRequest, VerifyPinResponse,
};

// Application state
struct AppState {
    pin: Option<String>,
    site_title: String,
    single_list: bool,
    allowed_origins: String,
    is_production: bool,
    data_file: String,
    // Brute force protection: IP -> (failed_attempts, last_attempt_time)
    login_attempts: RwLock<HashMap<String, (usize, Instant)>>,
}

const MAX_ATTEMPTS: usize = 5;
const LOCKOUT_TIME: Duration = Duration::from_secs(15 * 60); // 15 minutes

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let pin = std::env::var("DUMBDO_PIN").ok().filter(|p| !p.trim().is_empty());
    let site_title = std::env::var("DUMBDO_SITE_TITLE").unwrap_or_else(|_| "DumbDo".to_string());
    let single_list = std::env::var("SINGLE_LIST")
        .map(|val| val == "true")
        .unwrap_or(false);
    let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    let node_env = std::env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string());
    let is_production = node_env == "production";

    let data_dir = "data";
    let data_file = format!("{}/todos.json", data_dir);

    // Ensure data directory exists
    if let Err(e) = std::fs::create_dir_all(data_dir) {
        eprintln!("Failed to create data directory: {}", e);
    }

    // Initialize todos.json if missing
    if !std::path::Path::new(&data_file).exists() {
        if let Err(e) = std::fs::write(&data_file, "{}") {
            eprintln!("Failed to initialize todos file: {}", e);
        }
    }

    // Run migrations (ensure all todo items have IDs)
    run_todo_migrations(&data_file);

    let app_state = Arc::new(AppState {
        pin,
        site_title,
        single_list,
        allowed_origins: allowed_origins.clone(),
        is_production,
        data_file,
        login_attempts: RwLock::new(HashMap::new()),
    });

    // Start background task to clean up old lockouts
    let clean_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let mut attempts = clean_state.login_attempts.write().await;
            attempts.retain(|_, (_, last_time)| last_time.elapsed() < LOCKOUT_TIME);
        }
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    let cors = if allowed_origins == "*" {
        cors.allow_origin(Any)
    } else {
        // Parse comma separated origins
        let mut origins = Vec::new();
        for origin in allowed_origins.split(',') {
            if let Ok(parsed) = origin.trim().parse() {
                origins.push(parsed);
            }
        }
        cors.allow_origin(origins)
    };

    // Protected API routes
    let protected_routes = Router::new()
        .route("/todos", get(get_todos).post(save_todos))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Public API routes
    let api_routes = Router::new()
        .route("/pin-required", get(get_pin_required))
        .route("/verify-pin", post(verify_pin))
        .route("/config", get(get_config))
        .merge(protected_routes)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            origin_validation_middleware,
        ));

    // Router
    let app = Router::new()
        .nest("/api", api_routes)
        // Serve front-end assets from frontend/dist/assets
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .route("/favicon.svg", get(serve_favicon))
        .route("/manifest.json", get(serve_manifest))
        .route("/asset-manifest.json", get(serve_asset_manifest))
        .route("/service-worker.js", get(serve_service_worker))
        .fallback(serve_index)
        .layer(cors)
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("DumbDo/RustDo server running at http://localhost:{}", port);
    println!(
        "PIN protection: {}",
        if app_state.pin.is_some() {
            "enabled"
        } else {
            "disabled"
        }
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

// Ensure all existing tasks have a unique ID
fn run_todo_migrations(data_file: &str) {
    if let Ok(content) = std::fs::read_to_string(data_file) {
        if let Ok(mut lists) = serde_json::from_str::<TodoLists>(&content) {
            let mut updated = false;
            for items in lists.values_mut() {
                for item in items.iter_mut() {
                    if item.id.is_empty() {
                        item.id = generate_random_id();
                        updated = true;
                    }
                }
            }
            if updated {
                if let Ok(serialized) = serde_json::to_string_pretty(&lists) {
                    let temp_file = format!("{}.tmp", data_file);
                    if std::fs::write(&temp_file, serialized).is_ok() {
                        let _ = std::fs::rename(temp_file, data_file);
                        println!("Migration: assigned unique IDs to tasks.");
                    }
                }
            }
        }
    }
}

fn generate_random_id() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(9)
        .map(char::from)
        .collect()
}

// Timing-safe comparison using SHA-256
fn secure_compare(a: &str, b: &str) -> bool {
    let mut hasher_a = Sha256::new();
    hasher_a.update(a.as_bytes());
    let a_hash = hasher_a.finalize();

    let mut hasher_b = Sha256::new();
    hasher_b.update(b.as_bytes());
    let b_hash = hasher_b.finalize();

    let mut result = 0;
    for (x, y) in a_hash.iter().zip(b_hash.iter()) {
        result |= x ^ y;
    }
    result == 0
}

fn get_client_ip(connect_info: &ConnectInfo<SocketAddr>, headers: &HeaderMap) -> String {
    if let Some(cf_connecting_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip) = cf_connecting_ip.to_str() {
            return ip.to_string();
        }
    }
    if let Some(x_forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_list) = x_forwarded_for.to_str() {
            if let Some(ip) = ip_list.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    if let Some(x_real_ip) = headers.get("x-real-ip") {
        if let Ok(ip) = x_real_ip.to_str() {
            return ip.to_string();
        }
    }
    connect_info.ip().to_string()
}

// Origin validation middleware
async fn origin_validation_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if state.allowed_origins == "*" || !state.is_production {
        return next.run(request).await;
    }

    let origin = headers
        .get("origin")
        .or_else(|| headers.get("referer"))
        .and_then(|val| val.to_str().ok());

    if let Some(origin_str) = origin {
        let origin_norm = if let Ok(url) = url::Url::parse(origin_str) {
            url.origin().ascii_serialization()
        } else {
            origin_str.to_string()
        };

        let allowed = state.allowed_origins.split(',').any(|o| {
            let o_trimmed = o.trim();
            if let Ok(url) = url::Url::parse(o_trimmed) {
                url.origin().ascii_serialization() == origin_norm
            } else {
                o_trimmed == origin_norm
            }
        });

        if allowed {
            next.run(request).await
        } else {
            (StatusCode::FORBIDDEN, "Forbidden").into_response()
        }
    } else {
        (StatusCode::FORBIDDEN, "Forbidden").into_response()
    }
}

// Authentication check
fn is_authenticated(state: &AppState, cookie_jar: &CookieJar, headers: &HeaderMap) -> bool {
    let pin_env = match &state.pin {
        Some(p) => p,
        None => return true,
    };

    let provided_pin = cookie_jar
        .get("DUMBDO_PIN")
        .map(|c| c.value())
        .or_else(|| headers.get("x-pin").and_then(|h| h.to_str().ok()));

    if let Some(provided) = provided_pin {
        secure_compare(provided, pin_env)
    } else {
        false
    }
}

// Authentication middleware
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    cookie_jar: CookieJar,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if is_authenticated(&state, &cookie_jar, &headers) {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid PIN" }))).into_response()
    }
}

// Handlers
async fn get_pin_required(
    State(state): State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Json<PinRequiredResponse> {
    let client_ip = get_client_ip(&connect_info, &headers);
    let attempts = state.login_attempts.read().await;

    let (failed_count, last_attempt) = attempts
        .get(&client_ip)
        .cloned()
        .unwrap_or((0, Instant::now()));

    let locked = failed_count >= MAX_ATTEMPTS && last_attempt.elapsed() < LOCKOUT_TIME;
    let attempts_left = if locked {
        0
    } else if failed_count >= MAX_ATTEMPTS {
        MAX_ATTEMPTS
    } else {
        MAX_ATTEMPTS - failed_count
    };

    let lockout_minutes = if locked {
        let elapsed = last_attempt.elapsed();
        let remaining = LOCKOUT_TIME.saturating_sub(elapsed);
        (remaining.as_secs() + 59) / 60
    } else {
        0
    };

    Json(PinRequiredResponse {
        required: state.pin.is_some(),
        length: state.pin.as_ref().map(|p| p.len()).unwrap_or(4),
        locked,
        attempts_left,
        lockout_minutes,
    })
}

async fn verify_pin(
    State(state): State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    Json(payload): Json<VerifyPinRequest>,
) -> Response {
    let client_ip = get_client_ip(&connect_info, &headers);
    let pin_env = match &state.pin {
        Some(p) => p,
        None => {
            return (
                StatusCode::OK,
                Json(VerifyPinResponse {
                    valid: true,
                    error: None,
                    attempts_left: None,
                    locked: None,
                    lockout_minutes: None,
                }),
            )
                .into_response();
        }
    };

    // Check lockout status
    {
        let attempts = state.login_attempts.read().await;
        if let Some(&(failed_count, last_attempt)) = attempts.get(&client_ip) {
            if failed_count >= MAX_ATTEMPTS && last_attempt.elapsed() < LOCKOUT_TIME {
                let remaining = LOCKOUT_TIME.saturating_sub(last_attempt.elapsed());
                let minutes = (remaining.as_secs() + 59) / 60;
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(VerifyPinResponse {
                        valid: false,
                        error: Some(format!(
                            "Too many attempts. Please try again in {} minutes.",
                            minutes
                        )),
                        attempts_left: Some(0),
                        locked: Some(true),
                        lockout_minutes: Some(minutes),
                    }),
                )
                    .into_response();
            }
        }
    }

    // PIN length validation
    if payload.pin.len() < 4 || payload.pin.len() > 10 {
        let mut attempts = state.login_attempts.write().await;
        let entry = attempts.entry(client_ip).or_insert((0, Instant::now()));
        entry.0 += 1;
        entry.1 = Instant::now();
        let left = MAX_ATTEMPTS.saturating_sub(entry.0);

        return (
            StatusCode::UNAUTHORIZED,
            Json(VerifyPinResponse {
                valid: false,
                error: Some("PIN must be between 4 and 10 digits".to_string()),
                attempts_left: Some(left),
                locked: Some(left == 0),
                lockout_minutes: Some(if left == 0 { 15 } else { 0 }),
            }),
        )
            .into_response();
    }

    // Artificial delay to deter timing attacks
    let delay_ms = rand::thread_rng().gen_range(50..150);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    let valid = secure_compare(&payload.pin, pin_env);

    let mut attempts = state.login_attempts.write().await;
    if valid {
        attempts.remove(&client_ip);

        let cookie = Cookie::build(("DUMBDO_PIN", payload.pin))
            .http_only(true)
            .secure(state.is_production)
            .same_site(cookie::SameSite::Strict)
            .path("/")
            .build();

        let updated_jar = cookie_jar.add(cookie);

        (
            StatusCode::OK,
            updated_jar,
            Json(VerifyPinResponse {
                valid: true,
                error: None,
                attempts_left: None,
                locked: None,
                lockout_minutes: None,
            }),
        )
            .into_response()
    } else {
        let entry = attempts.entry(client_ip).or_insert((0, Instant::now()));
        entry.0 += 1;
        entry.1 = Instant::now();
        let left = MAX_ATTEMPTS.saturating_sub(entry.0);

        (
            StatusCode::UNAUTHORIZED,
            Json(VerifyPinResponse {
                valid: false,
                error: Some(format!(
                    "Invalid PIN. {} attempts remaining before lockout.",
                    left
                )),
                attempts_left: Some(left),
                locked: Some(left == 0),
                lockout_minutes: Some(if left == 0 { 15 } else { 0 }),
            }),
        )
            .into_response()
    }
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<SiteConfig> {
    Json(SiteConfig {
        site_title: state.site_title.clone(),
        single_list: state.single_list,
    })
}

async fn get_todos(State(state): State<Arc<AppState>>) -> Response {
    match tokio::fs::read_to_string(&state.data_file).await {
        Ok(content) => {
            let json: Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
            Json(json).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read todos").into_response(),
    }
}

async fn save_todos(State(state): State<Arc<AppState>>, Json(payload): Json<Value>) -> Response {
    let content = match serde_json::to_string_pretty(&payload) {
        Ok(c) => c,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JSON").into_response(),
    };

    let temp_file = format!("{}.tmp", state.data_file);
    if let Err(e) = tokio::fs::write(&temp_file, content).await {
        eprintln!("Failed to write to temp file: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save todos").into_response();
    }

    if let Err(e) = tokio::fs::rename(&temp_file, &state.data_file).await {
        eprintln!("Failed to rename temp file to data file: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save todos").into_response();
    }

    Json(serde_json::json!({ "success": true })).into_response()
}

// Serve static assets
async fn serve_index() -> Response {
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Frontend not compiled. Please run trunk build inside frontend/.",
        )
            .into_response(),
    }
}

async fn serve_favicon() -> Response {
    serve_static_file("frontend/dist/favicon.svg", "image/svg+xml").await
}

async fn serve_service_worker() -> Response {
    serve_static_file("frontend/dist/service-worker.js", "application/javascript").await
}

async fn serve_manifest(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
            }
        ],
        "orientation": "any"
    });
    Json(manifest)
}

// Helper to recursively collect files inside the Trunk output directory
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

async fn serve_asset_manifest() -> impl IntoResponse {
    let dist_path = Path::new("frontend/dist");
    let mut files = get_files_recursive(dist_path, dist_path);
    // Explicitly add routing base-paths to manifest if needed
    if !files.contains(&"/favicon.svg".to_string()) {
        files.push("/favicon.svg".to_string());
    }
    if !files.contains(&"/manifest.json".to_string()) {
        files.push("/manifest.json".to_string());
    }
    Json(files)
}

async fn serve_static_file(path: &str, content_type: &str) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, content_type)], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
