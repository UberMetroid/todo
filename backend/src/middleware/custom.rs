//! Axum middleware: thin wrappers over `shared_backend::middleware`.
//!
//! Every layer here is either a one-line re-export or a thin glue layer
//! that adapts to todo's per-route state. All actual security policy
//! lives in `shared_assets` so that beam / pad / todo / trace / grid
//! behave identically.

use axum::{
    Json,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use std::net::SocketAddr;

use crate::auth::secure_compare;
use crate::state::{AppState, SharedState, get_client_ip};

// ─────────────────────────── authentication ────────────────────────────

/// Returns `true` if the request is from an authenticated session or
/// presents a valid `x-pin` header.
pub async fn is_authenticated(
    state: &AppState,
    cookie_jar: &CookieJar,
    headers: &HeaderMap,
) -> bool {
    let pin_env = match &state.pin {
        Some(p) => p,
        None => return true, // public mode
    };

    let cookie_pin = cookie_jar.get("TODO_PIN").map(|c| c.value());
    let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

    match (cookie_pin, header_pin) {
        (Some(cookie), _) => state.active_sessions.read().await.contains(cookie),
        (None, Some(hdr)) => secure_compare(hdr.as_bytes(), pin_env.as_bytes()),
        (None, None) => false,
    }
}

/// Gate protected routes. On miss, returns `401 {"error": "Invalid PIN"}`.
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

// ────────────────────────────── rate limit ──────────────────────────────

/// Per-IP sliding-window rate limit. Delegates to `AppState::check_rate_limit`.
pub async fn rate_limit_middleware(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip = get_client_ip(&headers, addr, state.trust_proxy, &state.trusted_proxies);

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

// ──────────────────────────── origin check ─────────────────────────────

/// Reject cross-origin requests in production mode when an allowlist is
/// configured. In development, or when `ALLOWED_ORIGINS=*`, requests are
/// allowed unconditionally.
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

// ────────────────────── shared-assets layer re-exports ─────────────────
//
// These two functions match the `(Request, Next) -> Response` shape
// expected by `axum::middleware::from_fn`, so they can be installed
// without a `State` extractor. HSTS and title-injection need config, so
// they are installed in `main.rs` via `from_fn_with_state` directly —
// not wrapped here.

/// Re-export `shared_backend::middleware::security_headers_layer` for use
/// with `axum::middleware::from_fn(security_headers_middleware)`.
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    shared_backend::middleware::security_headers_layer(request, next).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_origin_strips_path() {
        assert_eq!(
            extract_origin("https://example.com/"),
            "https://example.com"
        );
        assert_eq!(
            extract_origin("https://example.com/path?q=1"),
            "https://example.com"
        );
        assert_eq!(extract_origin("example.com"), "example.com");
    }

    #[test]
    fn extract_origin_handles_no_scheme() {
        assert_eq!(extract_origin("localhost:4403"), "localhost:4403");
    }
}
