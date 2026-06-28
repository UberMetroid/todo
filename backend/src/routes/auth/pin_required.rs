use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use std::net::SocketAddr;

use super::super::PIN_MIN_LEN;
use crate::state::{SharedState, get_client_ip};
use shared_core::types::PinRequiredResponse;

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
        shared_backend::auth::is_locked_out(&client_ip, max_attempts, state.lockout_duration);
    let remaining_secs =
        shared_backend::auth::lockout_remaining_secs(&client_ip, state.lockout_duration);
    let attempts_left =
        shared_backend::auth::attempts_left(&client_ip, max_attempts, state.lockout_duration)
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
