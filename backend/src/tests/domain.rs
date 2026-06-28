use crate::auth::build_session_cookie_header;
use crate::types::TodoState;
use std::collections::HashMap;

#[test]
fn todo_state_envelope_roundtrip() {
    let mut lists = HashMap::new();
    lists.insert(
        "inbox".into(),
        vec![shared::TodoItem {
            id: "abc123".into(),
            text: "buy milk".into(),
            completed: false,
        }],
    );
    let state = TodoState { version: 7, lists };
    let json = serde_json::to_string(&state).unwrap();
    let back: TodoState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, back);
}

#[test]
fn todo_state_migrates_legacy_format() {
    let legacy = r#"{"inbox":[{"id":"a","text":"x","completed":false}]}"#;
    let (state, needs_rewrite) = TodoState::parse_with_migration(legacy).unwrap();
    assert_eq!(state.version, 1);
    assert!(needs_rewrite);
    assert_eq!(state.lists.len(), 1);
}

#[test]
fn todo_state_rejects_garbage() {
    assert!(TodoState::parse_with_migration("not json").is_err());
}

#[test]
fn session_cookie_has_security_attrs() {
    let h = build_session_cookie_header("deadbeef", 24, false);
    assert!(h.starts_with("TODO_PIN=deadbeef"));
    assert!(h.contains("HttpOnly"));
    assert!(h.contains("SameSite=Strict"));
    assert!(h.contains("Path=/"));
    assert!(h.contains("Max-Age=86400"));
    assert!(!h.contains("; Secure"));
}

#[test]
fn session_cookie_includes_secure_when_https() {
    let h = build_session_cookie_header("x", 1, true);
    assert!(h.contains("; Secure"));
    assert!(h.contains("Max-Age=3600"));
}
