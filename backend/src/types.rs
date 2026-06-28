//! Backend-only types that don't belong in the `shared_core` crate.
//!
//! Currently this is just [`TodoState`], the envelope around
//! [`shared_core::types::TodoLists`] that adds an
//! optimistic-concurrency version field. The on-disk shape of
//! `data/todos.json` is now this envelope instead of the raw list map.
//!
//! Auth/config wire types ([`shared_core::types::PinRequiredResponse`],
//! [`shared_core::types::SiteConfig`],
//! [`shared_core::types::VerifyPinRequest`],
//! [`shared_core::types::VerifyPinResponse`]) live in the
//! `shared_core` crate because the frontend also consumes them.

use serde::{Deserialize, Serialize};

/// On-disk envelope around [`shared_core::types::TodoLists`].
///
/// Wrapping the data adds optimistic-concurrency control: every save
/// must include the version it observed, and the server rejects with
/// 409 if the file was updated concurrently.
///
/// The on-disk JSON shape is:
/// ```json
/// { "version": 1, "lists": { "list_name": [...] } }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TodoState {
    /// Monotonically increasing counter, incremented on every
    /// successful save.
    #[serde(default)]
    pub version: u64,
    /// The actual list data.
    pub lists: shared_core::types::TodoLists,
}

impl TodoState {
    /// Parse a file that may be either:
    /// 1. The new envelope format `{ "version": N, "lists": {...} }`, or
    /// 2. The legacy raw map `{ "list_name": [...] }`.
    ///
    /// Returns `(state, needs_rewrite)` — `needs_rewrite` is true when
    /// the file was in legacy format and should be rewritten in the
    /// new format on the next successful save.
    pub fn parse_with_migration(content: &str) -> Result<(Self, bool), serde_json::Error> {
        // Try the envelope format first.
        if let Ok(state) = serde_json::from_str::<TodoState>(content) {
            return Ok((state, false));
        }
        // Fall back to the legacy raw map and migrate.
        let lists: shared_core::types::TodoLists = serde_json::from_str(content)?;
        Ok((Self { version: 1, lists }, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn envelope_roundtrip() {
        let mut lists: shared_core::types::TodoLists = HashMap::new();
        lists.insert(
            "inbox".into(),
            vec![shared_core::types::TodoItem {
                id: "abc".into(),
                text: "x".into(),
                completed: false,
            }],
        );
        let state = TodoState { version: 5, lists };
        let json = serde_json::to_string(&state).unwrap();
        let back: TodoState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, back);
    }

    #[test]
    fn envelope_migrates_legacy_format() {
        let legacy = r#"{"inbox":[{"id":"a","text":"x","completed":false}]}"#;
        let (state, needs_rewrite) = TodoState::parse_with_migration(legacy).unwrap();
        assert_eq!(state.version, 1);
        assert!(needs_rewrite);
        assert_eq!(state.lists.len(), 1);
        assert_eq!(state.lists["inbox"][0].id, "a");
    }

    #[test]
    fn envelope_recognises_envelope_format() {
        let envelope = r#"{"version":7,"lists":{"x":[]}}"#;
        let (state, needs_rewrite) = TodoState::parse_with_migration(envelope).unwrap();
        assert_eq!(state.version, 7);
        assert!(!needs_rewrite);
    }

    #[test]
    fn envelope_rejects_garbage() {
        let garbage = "not json at all";
        assert!(TodoState::parse_with_migration(garbage).is_err());
    }
}