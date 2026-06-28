use crate::auth::{generate_random_id, generate_session_id};
use crate::routes;
use crate::state::AppState;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use shared::{PinRequiredResponse, SiteConfig, VerifyPinRequest};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

fn test_state() -> Arc<AppState> {
    Arc::new(AppState {
        pin: Some("12345678".into()),
        site_title: "TestDo".into(),
        single_list: false,
        allowed_origins: "*".into(),
        is_production: false,
        data_file: "test_todos.json".into(),
        asset_manifest: vec![],
        max_attempts: 5,
        lockout_duration: std::time::Duration::from_secs(900),
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

#[test]
fn session_id_format() {
    let id = generate_session_id();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn random_id_format() {
    let id = generate_random_id();
    assert_eq!(id.len(), 9);
    assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn random_ids_are_unique() {
    let a = generate_random_id();
    let b = generate_random_id();
    assert_ne!(a, b);
}

#[tokio::test]
async fn get_config_returns_state_values() {
    let state = test_state();
    let config: SiteConfig = routes::get_config(State(state)).await.0;
    assert_eq!(config.site_title, "TestDo");
    assert!(!config.single_list);
    assert!(config.show_version);
    assert!(config.show_github);
}

#[tokio::test]
async fn get_pin_required_reports_correct_values() {
    let state = test_state();
    let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();
    let res: PinRequiredResponse = routes::get_pin_required(State(state), connect_info, headers)
        .await
        .0;
    assert!(res.required);
    assert_eq!(res.length, 8);
    assert!(!res.locked);
    assert_eq!(res.attempts_left, 5);
}

#[tokio::test]
async fn verify_pin_correct_returns_ok() {
    let state = test_state();
    let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();
    let jar = CookieJar::new();
    let req = VerifyPinRequest {
        pin: "12345678".into(),
    };
    let res = routes::verify_pin(State(state), connect_info, headers, jar, Json(req)).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_pin_wrong_returns_401() {
    let state = test_state();
    let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 2], 12345)));
    let headers = HeaderMap::new();
    let jar = CookieJar::new();
    let req = VerifyPinRequest {
        pin: "87654321".into(),
    };
    let res = routes::verify_pin(State(state), connect_info, headers, jar, Json(req)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn verify_pin_short_returns_400_without_incrementing() {
    let state = test_state();
    let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 3], 12345)));
    let headers = HeaderMap::new();
    let jar = CookieJar::new();
    let req = VerifyPinRequest { pin: "1".into() };
    let res = routes::verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let left = shared_assets::auth::attempts_left(
        "10.0.0.3",
        state.max_attempts as u32,
        state.lockout_duration,
    );
    assert_eq!(left, 5, "format error must not consume an attempt");
}

#[tokio::test]
async fn verify_pin_lockout_after_max_attempts() {
    let state = test_state();
    let connect_info = ConnectInfo(SocketAddr::from(([10, 0, 0, 4], 12345)));
    let headers = HeaderMap::new();
    for i in 0..5 {
        let jar = CookieJar::new();
        let req = VerifyPinRequest {
            pin: format!("wrong{i}"),
        };
        let _ = routes::verify_pin(
            State(state.clone()),
            connect_info,
            headers.clone(),
            jar,
            Json(req),
        )
        .await;
    }
    let jar = CookieJar::new();
    let req = VerifyPinRequest {
        pin: "12345678".into(),
    };
    let res = routes::verify_pin(State(state.clone()), connect_info, headers, jar, Json(req)).await;
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limit_allows_under_threshold() {
    let state = test_state();
    for _ in 0..50 {
        assert!(state.check_rate_limit("5.5.5.5").await);
    }
}

#[tokio::test]
async fn rate_limit_blocks_over_threshold() {
    let state = test_state();
    for _ in 0..100 {
        assert!(state.check_rate_limit("5.5.5.6").await);
    }
    assert!(!state.check_rate_limit("5.5.5.6").await);
}

#[test]
fn client_ip_no_proxy_returns_socket() {
    let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
    let headers = HeaderMap::new();
    let ip = crate::state::get_client_ip(&headers, socket, false, &[]);
    assert_eq!(ip, "10.0.0.1");
}

#[test]
fn client_ip_proxy_without_list_ignores_xff() {
    let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
    let ip = crate::state::get_client_ip(&headers, socket, true, &[]);
    assert_eq!(ip, "10.0.0.1");
}

#[test]
fn client_ip_proxy_with_allowlist_honors_xff() {
    let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
    let trusted: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
    let ip = crate::state::get_client_ip(&headers, socket, true, &[trusted]);
    assert_eq!(ip, "1.2.3.4");
}
