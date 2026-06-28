pub mod auth;
pub mod config;
pub mod todos;

pub use auth::{get_pin_required, logout, verify_pin};
pub use config::get_config;
pub use todos::{get_todos, save_todos};

pub const PIN_MIN_LEN: usize = 4;
pub const PIN_MAX_LEN: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SharedState;
    use axum::{
        Json,
        extract::{ConnectInfo, State},
        http::{HeaderMap, StatusCode},
    };
    use axum_extra::extract::cookie::CookieJar;
    use shared_core::types::VerifyPinRequest;
    use std::collections::HashMap;
    use std::net::SocketAddr;
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
        let left = shared_backend::auth::attempts_left(
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
        let left = shared_backend::auth::attempts_left(
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
        let left = shared_backend::auth::attempts_left(
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
