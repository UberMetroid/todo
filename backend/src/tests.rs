use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use shared::VerifyPinRequest;

#[test]
fn test_secure_compare() {
    assert!(auth::secure_compare("1234", "1234"));
    assert!(!auth::secure_compare("1234", "1235"));
    assert!(!auth::secure_compare("1234", "12345"));
    assert!(!auth::secure_compare("12345", "1234"));
    assert!(auth::secure_compare("", ""));
}

#[test]
fn test_generate_random_id() {
    let id1 = auth::generate_random_id();
    let id2 = auth::generate_random_id();
    assert_eq!(id1.len(), 9);
    assert_eq!(id2.len(), 9);
    assert_ne!(id1, id2);
    for c in id1.chars() {
        assert!(c.is_ascii_alphanumeric());
    }
}

#[tokio::test]
async fn test_get_config_handler() {
    let state = Arc::new(state::AppState {
        pin: Some("1234".to_string()),
        site_title: "TestDo".to_string(),
        single_list: true,
        allowed_origins: "*".to_string(),
        is_production: false,
        data_file: "test_todos.json".to_string(),
        asset_manifest: vec![],
        login_attempts: RwLock::new(HashMap::new()),
    });

    let config = handlers::get_config(State(state)).await;
    assert_eq!(config.site_title, "TestDo");
    assert!(config.single_list);
}

#[tokio::test]
async fn test_get_pin_required_handler() {
    let state = Arc::new(state::AppState {
        pin: Some("1234".to_string()),
        site_title: "TestDo".to_string(),
        single_list: false,
        allowed_origins: "*".to_string(),
        is_production: false,
        data_file: "test_todos.json".to_string(),
        asset_manifest: vec![],
        login_attempts: RwLock::new(HashMap::new()),
    });

    let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();

    let res = handlers::get_pin_required(State(state.clone()), connect_info, headers).await;
    assert!(res.required);
    assert_eq!(res.length, 4);
    assert!(!res.locked);
    assert_eq!(res.attempts_left, 5);
}

#[tokio::test]
async fn test_verify_pin_handler() {
    let state = Arc::new(state::AppState {
        pin: Some("1234".to_string()),
        site_title: "TestDo".to_string(),
        single_list: false,
        allowed_origins: "*".to_string(),
        is_production: false,
        data_file: "test_todos.json".to_string(),
        asset_manifest: vec![],
        login_attempts: RwLock::new(HashMap::new()),
    });

    let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();
    let jar = CookieJar::new();

    // Verify correct PIN
    let req = VerifyPinRequest { pin: "1234".to_string() };
    let res = handlers::verify_pin(State(state.clone()), connect_info.clone(), headers.clone(), jar.clone(), Json(req)).await;
    assert_eq!(res.status(), StatusCode::OK);

    // Verify incorrect PIN
    let req_wrong = VerifyPinRequest { pin: "5678".to_string() };
    let res_wrong = handlers::verify_pin(State(state.clone()), connect_info, headers, jar, Json(req_wrong)).await;
    assert_eq!(res_wrong.status(), StatusCode::UNAUTHORIZED);
}
