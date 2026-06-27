//! Shared types between the `todo` frontend and backend.
//!
//! This crate exists as the **wire-format contract** between the Yew
//! frontend and the Axum backend. It does **not** host any business
//! logic — that lives in `backend/src/`.
//!
//! Types are organised into two groups:
//!
//! - **Domain types** ([`TodoItem`], [`TodoLists`]) — the actual user
//!   data. These are what flows through the JSON endpoints and is
//!   persisted to `data/todos.json`.
//! - **API contract types** ([`SiteConfig`], [`PinRequiredResponse`],
//!   [`VerifyPinRequest`], [`VerifyPinResponse`]) — the shapes used by
//!   the auth and config endpoints. The frontend consumes these to
//!   render the login screen and site header.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

// ──────────────────────────── domain types ─────────────────────────────

/// A single todo item, as stored in `data/todos.json` and rendered in
/// the frontend.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    /// Stable identifier, generated client-side or by the migration in
    /// `auth::run_todo_migrations`. Used as the React/Yew key so that
    /// reorder/edit/delete operations are O(1).
    #[serde(default)]
    pub id: String,
    /// The task text. May contain any UTF-8.
    pub text: String,
    /// Whether the task has been completed.
    pub completed: bool,
}

/// On-disk and over-the-wire representation: `list_name -> ordered tasks`.
///
/// Persisted to `data/todos.json` as a top-level JSON object. From v2.2.0
/// onward, the file is wrapped in a [`TodoState`] envelope on the
/// backend for optimistic concurrency; the envelope wraps a
/// [`TodoLists`] payload of this shape.
pub type TodoLists = HashMap<String, Vec<TodoItem>>;

// ──────────────────────── API contract types ───────────────────────────

/// Public site configuration. Returned by `GET /api/config` and used by
/// the frontend to render the header, theme picker, etc.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    pub site_title: String,
    pub single_list: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    #[serde(default = "default_true")]
    pub show_version: bool,
    #[serde(default = "default_true")]
    pub show_github: bool,
}

/// Status of PIN authentication on the server. Returned by
/// `GET /api/pin-required` so the login screen can render the correct
/// prompts and disable itself when no PIN is configured.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PinRequiredResponse {
    pub required: bool,
    pub length: usize,
    pub locked: bool,
    pub attempts_left: usize,
    pub lockout_minutes: u64,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    #[serde(default = "default_true")]
    pub show_version: bool,
    #[serde(default = "default_true")]
    pub show_github: bool,
}

/// Request body for `POST /api/verify-pin`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VerifyPinRequest {
    pub pin: String,
}

/// Response body for `POST /api/verify-pin`.
///
/// `error`, `attempts_left`, `locked`, `lockout_minutes` are populated
/// on failure and omitted on success so that the success case is the
/// minimal `{"valid": true}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPinResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts_left: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockout_minutes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_item_default_id_is_empty() {
        let item = TodoItem {
            id: String::new(),
            text: "x".into(),
            completed: false,
        };
        assert_eq!(item.id, "");
    }

    #[test]
    fn todo_lists_roundtrip() {
        let mut lists: TodoLists = HashMap::new();
        lists.insert(
            "inbox".to_string(),
            vec![TodoItem {
                id: "abc123".into(),
                text: "buy milk".into(),
                completed: false,
            }],
        );
        let json = serde_json::to_string(&lists).unwrap();
        let back: TodoLists = serde_json::from_str(&json).unwrap();
        assert_eq!(lists, back);
    }

    #[test]
    fn verify_pin_response_success_is_minimal() {
        let resp = VerifyPinResponse {
            valid: true,
            error: None,
            attempts_left: None,
            locked: None,
            lockout_minutes: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"valid":true}"#);
    }

    #[test]
    fn verify_pin_response_failure_includes_details() {
        let resp = VerifyPinResponse {
            valid: false,
            error: Some("bad".into()),
            attempts_left: Some(3),
            locked: Some(false),
            lockout_minutes: Some(0),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"attemptsLeft\":3"));
        assert!(json.contains("\"locked\":false"));
    }

    #[test]
    fn site_config_defaults_show_flags_true() {
        let json =
            r#"{"siteTitle":"x","singleList":false,"enableThemes":false,"enablePrint":false}"#;
        let cfg: SiteConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.show_version);
        assert!(cfg.show_github);
    }
}
