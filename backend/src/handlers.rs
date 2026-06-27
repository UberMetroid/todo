//! HTTP handlers for the `todo` backend.
//!
//! These handlers delegate PIN-attempt tracking to
//! `shared_assets::auth::attempts` and IP resolution to
//! `shared_assets::server::ip`. They also implement the
//! optimistic-concurrency envelope for `data/todos.json`.

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::{net::SocketAddr, time::Duration};

use crate::auth::{build_session_cookie_header, secure_compare};
use crate::state::{SharedState, get_client_ip};
use crate::types::TodoState;
use shared::{PinRequiredResponse, SiteConfig, VerifyPinRequest, VerifyPinResponse};

/// Length bounds for a user-supplied PIN, matching shared-assets's
/// `ServerConfig::from_env` policy.
const PIN_MIN_LEN: usize = 4;
const PIN_MAX_LEN: usize = 64;

/// Default lockout window when the request does not yet have one pinned
/// to a specific IP (used only in unit tests; production always supplies
/// a duration).
#[cfg(test)]
#[allow(dead_code)]
const DEFAULT_LOCKOUT: Duration = Duration::from_secs(15 * 60);

pub async fn get_pin_required(
    State(state): State<SharedState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Json<PinRequiredResponse> {
    let client_ip = get_client_ip(
        &headers,
        connect_info.0,
        state.trust_proxy,
        &state.trusted_proxies,
    );
    let max_attempts = state.max_attempts as u32;

    let locked =
        shared_assets::auth::is_locked_out(&client_ip, max_attempts, state.lockout_duration);
    let remaining_secs =
        shared_assets::auth::lockout_remaining_secs(&client_ip, state.lockout_duration);
    let attempts_left =
        shared_assets::auth::attempts_left(&client_ip, max_attempts, state.lockout_duration)
            as usize;

    let lockout_minutes = if locked {
        remaining_secs.div_ceil(60)
    } else {
        0
    };

    Json(PinRequiredResponse {
        required: state.pin.is_some(),
        length: state.pin.as_ref().map(|p| p.len()).unwrap_or(PIN_MIN_LEN),
        locked,
        attempts_left,
        lockout_minutes,
        enable_translation: state.enable_translation,
        enable_themes: state.enable_themes,
        enable_print: state.enable_print,
        show_version: state.show_version,
        show_github: state.show_github,
    })
}

pub async fn verify_pin(
    State(state): State<SharedState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    Json(payload): Json<VerifyPinRequest>,
) -> Response {
    let client_ip = get_client_ip(
        &headers,
        connect_info.0,
        state.trust_proxy,
        &state.trusted_proxies,
    );
    let max_attempts = state.max_attempts as u32;
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

    // 1. Lockout check — return 429 without inspecting the PIN.
    if shared_assets::auth::is_locked_out(&client_ip, max_attempts, state.lockout_duration) {
        let remaining =
            shared_assets::auth::lockout_remaining_secs(&client_ip, state.lockout_duration);
        let minutes = remaining.div_ceil(60);
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

    // 2. Format check — return 400 WITHOUT incrementing the attempt
    //    counter. Format errors are not brute-force attempts.
    if payload.pin.len() < PIN_MIN_LEN || payload.pin.len() > PIN_MAX_LEN {
        return (
            StatusCode::BAD_REQUEST,
            Json(VerifyPinResponse {
                valid: false,
                error: Some(format!(
                    "PIN must be between {PIN_MIN_LEN} and {PIN_MAX_LEN} characters"
                )),
                attempts_left: None,
                locked: None,
                lockout_minutes: None,
            }),
        )
            .into_response();
    }

    // 3. Constant-time comparison. Tiny randomised delay to flatten
    //    timing-side-channels between match and mismatch on online
    //    attacks; constant_time_eq itself prevents micro-timing leaks
    //    from the comparison itself.
    let delay_ms = {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        50 + (seed % 101)
    };
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    let valid = secure_compare(payload.pin.as_bytes(), pin_env.as_bytes());

    if valid {
        // Success — clear lockout counter, issue session cookie.
        shared_assets::auth::reset_attempts(&client_ip);

        let session_id = crate::auth::generate_session_id();
        state
            .active_sessions
            .write()
            .await
            .insert(session_id.clone());

        let is_secure = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or(false);

        let cookie_value =
            build_session_cookie_header(&session_id, state.cookie_max_age_hours, is_secure);
        let cookie = Cookie::build(("TODO_PIN", session_id))
            .http_only(true)
            .secure(is_secure)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .path("/")
            .build();

        let updated_jar = cookie_jar.add(cookie);

        // We don't need the raw header string, but log it for diagnostics.
        tracing::debug!(target: "auth", "session issued: cookie={cookie_value}");

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
        // Failed comparison — increment counter. If this is the attempt
        // that crosses `max_attempts`, the next call will see the
        // lockout.
        let attempt = shared_assets::auth::record_attempt(&client_ip);
        let left = max_attempts.saturating_sub(attempt.count) as usize;

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

pub async fn get_config(State(state): State<SharedState>) -> Json<SiteConfig> {
    Json(SiteConfig {
        site_title: state.site_title.clone(),
        single_list: state.single_list,
        enable_themes: state.enable_themes,
        enable_print: state.enable_print,
        show_version: state.show_version,
        show_github: state.show_github,
    })
}

/// Read the todos file. Returns the envelope form `{ version, lists }`.
/// Migrates legacy plain-map format transparently and rewrites the file
/// in envelope form on the next save.
pub async fn get_todos(State(state): State<SharedState>) -> Response {
    let data_file = state.data_file.clone();

    // Run the read+parse on a blocking thread — file IO should not
    // block the executor.
    let read_result = tokio::task::spawn_blocking(move || {
        let content = std::fs::read_to_string(&data_file)?;
        let (todo_state, needs_rewrite) = TodoState::parse_with_migration(&content)
            .map_err(|e| format!("data file is corrupt: {e}"))?;
        Ok::<(TodoState, bool), Box<dyn std::error::Error + Send + Sync>>((
            todo_state,
            needs_rewrite,
        ))
    })
    .await;

    match read_result {
        Ok(Ok((todo_state, _needs_rewrite))) => Json(todo_state).into_response(),
        Ok(Err(msg)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "{msg}. Please restore from `data/todos.json.bak` or contact the administrator."
            ),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read todos: {join_err}"),
        )
            .into_response(),
    }
}

/// Save the todos file with optimistic-concurrency control.
///
/// Body shape:
/// ```json
/// { "version": N, "lists": { "list_name": [...] } }
/// ```
///
/// If `version` is omitted (legacy clients), defaults to `0`, which
/// means concurrent overwrites are possible — but the response still
/// returns the new version so clients can opt in to versioning.
pub async fn save_todos(
    State(state): State<SharedState>,
    Json(payload): Json<TodoState>,
) -> Response {
    let data_file = state.data_file.clone();

    // 1. Read the current file to compare versions.
    let current_version: u64 = match tokio::fs::read_to_string(&data_file).await {
        Ok(content) => match TodoState::parse_with_migration(&content) {
            Ok((s, _)) => s.version,
            Err(_) => 0, // Treat corrupt file as version 0; rewrite will heal it.
        },
        Err(_) => 0,
    };

    // 2. Check optimistic-concurrency. A save is accepted only if the
    //    client's observed version matches the current file version.
    if payload.version != current_version && current_version != 0 {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "version_conflict",
                "current_version": current_version,
                "your_version": payload.version,
            })),
        )
            .into_response();
    }

    // 3. Build the new state with version = current + 1.
    let new_state = TodoState {
        version: current_version + 1,
        lists: payload.lists,
    };
    // Capture the new version *before* moving `new_state` into the
    // blocking write below — we need it for the response.
    let new_version = new_state.version;

    // 4. Write atomically: backup current → write to .tmp → rename.
    //    On failure, leave the original file untouched.
    let write_res = tokio::task::spawn_blocking(move || {
        use std::fs::{self, File};
        use std::io::BufWriter;

        // Backup current file (best-effort; ignore failure if file
        // doesn't exist yet).
        if let Ok(content) = fs::read_to_string(&data_file) {
            let backup = format!("{data_file}.bak");
            let _ = fs::write(&backup, content);
        }

        let temp_file = format!("{data_file}.tmp");
        let file = File::create(&temp_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &new_state)?;
        fs::rename(&temp_file, &data_file)?;
        Ok::<(), std::io::Error>(())
    })
    .await;

    match write_res {
        Ok(Ok(())) => Json(serde_json::json!({
            "success": true,
            "version": new_version,
        }))
        .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save todos: {e}"),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save todos: {join_err}"),
        )
            .into_response(),
    }
}

pub async fn logout(cookie_jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    if let Some(cookie) = cookie_jar.get("TODO_PIN") {
        state.active_sessions.write().await.remove(cookie.value());
    }
    let cookie = Cookie::build(("TODO_PIN", ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .build();
    (StatusCode::OK, cookie_jar.remove(cookie))
}

// Re-export the helpers we used from shared_assets so the imports in
// this module stay short.

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn test_state() -> SharedState {
        Arc::new(crate::state::AppState {
            pin: Some("1234".into()),
            site_title: "T".into(),
            single_list: false,
            allowed_origins: "*".into(),
            is_production: false,
            data_file: "test_todos.json".into(),
            asset_manifest: vec![],
            max_attempts: 5,
            lockout_duration: std::time::Duration::from_secs(15 * 60),
            enable_translation: false,
            enable_themes: false,
            enable_print: false,
            show_version: true,
            show_github: true,
            trust_proxy: false,
            trusted_proxies: vec![],
            cookie_max_age_hours: 24,
            active_sessions: RwLock::new(std::collections::HashSet::new()),
            rate_limiter: RwLock::new(HashMap::new()),
        })
    }

    #[tokio::test]
    async fn pin_required_reports_required() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
        let headers = HeaderMap::new();
        let res = get_pin_required(State(state), connect_info, headers).await;
        assert!(res.required);
        assert_eq!(res.length, 4);
        assert!(!res.locked);
        assert_eq!(res.attempts_left, 5);
    }

    #[tokio::test]
    async fn verify_pin_correct_passes() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
        let headers = HeaderMap::new();
        let jar = CookieJar::new();
        let req = VerifyPinRequest { pin: "1234".into() };
        let res = verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn verify_pin_wrong_returns_unauthorized() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
        let headers = HeaderMap::new();
        let jar = CookieJar::new();
        let req = VerifyPinRequest { pin: "5678".into() };
        let res = verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn verify_pin_short_does_not_increment_counter() {
        // Use a unique IP via the headers.
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 99], 12345)));
        let headers = HeaderMap::new();
        let jar = CookieJar::new();
        let req = VerifyPinRequest { pin: "1".into() };
        let res = verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // Counter should still be 0 for this IP — no failed attempts
        // recorded for a format error.
        let left = shared_assets::auth::attempts_left(
            "10.0.0.99",
            state.max_attempts as u32,
            state.lockout_duration,
        );
        assert_eq!(left, 5, "format error must not consume an attempt");
    }

    #[tokio::test]
    async fn verify_pin_wrong_does_increment_counter() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 100], 12345)));
        let headers = HeaderMap::new();
        let jar = CookieJar::new();
        let req = VerifyPinRequest { pin: "5678".into() };
        let res = verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let left = shared_assets::auth::attempts_left(
            "10.0.0.100",
            state.max_attempts as u32,
            state.lockout_duration,
        );
        assert_eq!(left, 4, "wrong PIN must consume an attempt");
    }

    #[tokio::test]
    async fn verify_pin_correct_clears_counter() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 101], 12345)));
        let headers = HeaderMap::new();
        // First, fail.
        let jar = CookieJar::new();
        let _ = verify_pin(
            State(state.clone()),
            connect_info,
            headers.clone(),
            jar,
            Json(VerifyPinRequest { pin: "5678".into() }),
        )
        .await;
        // Then succeed.
        let jar2 = CookieJar::new();
        let _ = verify_pin(
            State(state.clone()),
            connect_info,
            headers,
            jar2,
            Json(VerifyPinRequest { pin: "1234".into() }),
        )
        .await;
        let left = shared_assets::auth::attempts_left(
            "10.0.0.101",
            state.max_attempts as u32,
            state.lockout_duration,
        );
        assert_eq!(left, 5, "successful PIN must reset the counter");
    }

    #[tokio::test]
    async fn get_config_returns_state() {
        let state = test_state();
        let config = get_config(State(state)).await;
        assert_eq!(config.site_title, "T");
        assert!(!config.single_list);
    }

    #[tokio::test]
    async fn logout_clears_session() {
        let state = test_state();
        let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 102], 12345)));
        let headers = HeaderMap::new();
        let mut jar = CookieJar::new();
        let _ = verify_pin(
            State(state.clone()),
            connect_info,
            headers,
            jar.clone(),
            Json(VerifyPinRequest { pin: "1234".into() }),
        )
        .await;
        // Capture the session id that verify_pin just minted so we can
        // put it on the jar that logout will read. (In production,
        // the Set-Cookie header from the response is what the browser
        // would persist; the test bypasses the HTTP plumbing.)
        let session_id = state
            .active_sessions
            .read()
            .await
            .iter()
            .next()
            .cloned()
            .expect("verify_pin should have inserted a session");
        jar = jar.add(
            axum_extra::extract::cookie::Cookie::build(("TODO_PIN", session_id))
                .path("/")
                .http_only(true)
                .build(),
        );
        // Now logout.
        let _ = logout(jar, State(state.clone())).await;
        assert!(state.active_sessions.read().await.is_empty());
    }
}
