use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde_json::Value;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use crate::auth::{secure_compare, MAX_ATTEMPTS};
use crate::state::{get_client_ip, SharedState};
use shared::{PinRequiredResponse, SiteConfig, VerifyPinRequest, VerifyPinResponse};

const LOCKOUT_TIME: Duration = Duration::from_secs(15 * 60); // 15 minutes

pub async fn get_pin_required(
    State(state): State<SharedState>,
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

pub async fn verify_pin(
    State(state): State<SharedState>,
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

    use rand::Rng;
    let delay_ms = rand::thread_rng().gen_range(50..150);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    let valid = secure_compare(&payload.pin, pin_env);

    let mut attempts = state.login_attempts.write().await;
    if valid {
        attempts.remove(&client_ip);

        let cookie = Cookie::build(("RUSTDO_PIN", payload.pin))
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

pub async fn get_config(State(state): State<SharedState>) -> Json<SiteConfig> {
    Json(SiteConfig {
        site_title: state.site_title.clone(),
        single_list: state.single_list,
    })
}

pub async fn get_todos(State(state): State<SharedState>) -> Response {
    match tokio::fs::read_to_string(&state.data_file).await {
        Ok(content) => {
            let json: Value =
                serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
            Json(json).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read todos").into_response(),
    }
}

pub async fn save_todos(State(state): State<SharedState>, Json(payload): Json<Value>) -> Response {
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
