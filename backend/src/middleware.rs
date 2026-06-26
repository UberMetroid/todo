use axum::{
    extract::{State, ConnectInfo},
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use std::net::SocketAddr;

use crate::auth::secure_compare;
use crate::state::{AppState, SharedState, get_client_ip};

pub async fn is_authenticated(state: &AppState, cookie_jar: &CookieJar, headers: &HeaderMap) -> bool {
    let pin_env = match &state.pin {
        Some(p) => p,
        None => return true,
    };

    let cookie_pin = cookie_jar
        .get("TODO_PIN")
        .or_else(|| cookie_jar.get("ADAM_PIN"))
        .map(|c| c.value());
    let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

    match (cookie_pin, header_pin) {
        (Some(cookie), _) => state.active_sessions.read().await.contains(cookie),
        (None, Some(hdr)) => secure_compare(hdr, pin_env),
        (None, None) => false,
    }
}

pub async fn auth_middleware(
    State(state): State<SharedState>,
    cookie_jar: CookieJar,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if is_authenticated(&state, &cookie_jar, &headers).await {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid PIN" })),
        )
            .into_response()
    }
}

pub async fn rate_limit_middleware(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip = get_client_ip(&ConnectInfo(addr), &headers);

    if !state.check_rate_limit(&ip).await {
        let body = serde_json::json!({
            "error": "Too many requests. Please slow down."
        });
        let mut response = axum::response::Json(body).into_response();
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        return response;
    }

    next.run(request).await
}

pub async fn origin_validation_middleware(
    State(state): State<SharedState>,
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
        let origin_norm = extract_origin(origin_str);

        let allowed = state
            .allowed_origins
            .split(',')
            .any(|o| extract_origin(o.trim()) == origin_norm);

        if allowed {
            next.run(request).await
        } else {
            (StatusCode::FORBIDDEN, "Forbidden").into_response()
        }
    } else {
        (StatusCode::FORBIDDEN, "Forbidden").into_response()
    }
}

fn extract_origin(url: &str) -> &str {
    if let Some(pos) = url.find("://") {
        let start = pos + 3;
        let rest = &url[start..];
        if let Some(end) = rest.find('/') {
            &url[..start + end]
        } else {
            url
        }
    } else {
        url.trim()
    }
}

pub async fn security_headers_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        "X-Frame-Options",
        axum::http::header::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "X-Content-Type-Options",
        axum::http::header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "Referrer-Policy",
        axum::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy", 
        axum::http::header::HeaderValue::from_static(
            "default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; img-src 'self' data: blob: https:; connect-src 'self' ws: wss: http: https:; font-src 'self'; manifest-src 'self';"
        )
    );

    response
}
