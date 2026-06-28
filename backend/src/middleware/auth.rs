//! PIN-related helpers for the `todo` backend.
//!
//! The PIN verification, lockout, and rate-limit logic is delegated to
//! `shared_backend::auth::attempts` and `shared_backend::server::ip` so that
//! every UberMetroid companion app uses the same primitives.
//!
//! This module retains only the **todo-specific** helpers:
//!
//! - Session ID generation (todo uses random session IDs stored in
//!   `active_sessions`, not the raw PIN in the cookie — this avoids
//!   leaking the PIN if the cookie jar is dumped).
//! - Random 9-char task ID generation.
//! - File-format migrations for `data/todos.json`.
//! - A `TODO_PIN`-named cookie helper that uses the same security
//!   attributes as `shared_backend::auth::session::build_set_cookie_header`.

use shared_core::types::TodoLists;
use std::fs::File;
use std::io::Read;

/// Re-export the constant-time comparison from `shared-assets`'s
/// dependency. We deliberately do NOT roll our own.
pub use constant_time_eq::constant_time_eq as secure_compare;

/// Cryptographically-random 32-character session ID, hex-encoded.
///
/// Used as the cookie value after a successful PIN verification. Stored
/// server-side in `AppState::active_sessions` so that the cookie alone is
/// not sufficient to authenticate (the PIN itself is never stored on the
/// client).
pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 16];
    if let Ok(mut file) = File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    } else {
        // Fallback: hash the current nanosecond clock. Not ideal, but
        // better than a constant — and the application is running in a
        // container where /dev/urandom should always exist.
        let random_val = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(random_val.to_string().as_bytes());
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Random 9-character alphanumeric task ID, used to give every task a
/// stable identity across edits and reordering.
pub fn generate_random_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 9];

    if let Ok(mut file) = File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    } else {
        // Fallback LCG using system time as seed.
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        for byte in &mut bytes {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (seed >> 32) as u8;
        }
    }

    bytes.iter_mut().for_each(|b| {
        *b = CHARSET[(*b as usize) % CHARSET.len()];
    });

    String::from_utf8_lossy(&bytes).into_owned()
}

/// Build a `Set-Cookie` header value for a successful PIN session.
///
/// Cookie **name** is `TODO_PIN` (preserved for backward compatibility
/// with existing installs; changing it would invalidate every existing
/// session). Cookie **value** is a random session ID, never the PIN.
///
/// Security attributes match `shared_backend::auth::session::build_set_cookie_header`:
///
/// - `HttpOnly` — not readable from JavaScript
/// - `SameSite=Strict` — not sent on cross-site navigations
/// - `Secure` — only sent over HTTPS, when the request is HTTPS
/// - `Max-Age` — from `ServerConfig::cookie_max_age_hours`
/// - `Path=/` — sent on every route
pub fn build_session_cookie_header(value: &str, max_age_hours: i64, is_secure: bool) -> String {
    let max_age = max_age_hours * 3600;
    let secure_flag = if is_secure { "; Secure" } else { "" };
    format!("TODO_PIN={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={max_age}{secure_flag}")
}

/// Migrate any task items that lack an `id` field. Writes back in-place
/// atomically (temp + rename) if any IDs were assigned.
pub fn run_todo_migrations(data_file: &str) {
    if let Ok(content) = std::fs::read_to_string(data_file)
        && let Ok(mut lists) = serde_json::from_str::<TodoLists>(&content)
    {
        let mut updated = false;
        for items in lists.values_mut() {
            for item in items.iter_mut() {
                if item.id.is_empty() {
                    item.id = generate_random_id();
                    updated = true;
                }
            }
        }
        if updated && let Ok(serialized) = serde_json::to_string_pretty(&lists) {
            let temp_file = format!("{}.tmp", data_file);
            if std::fs::write(&temp_file, serialized).is_ok() {
                let _ = std::fs::rename(temp_file, data_file);
                println!("Migration: assigned unique IDs to tasks.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_32_hex_chars() {
        let id = generate_session_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn session_ids_are_unique() {
        let a = generate_session_id();
        let b = generate_session_id();
        assert_ne!(a, b);
    }

    #[test]
    fn task_id_is_9_alphanumeric_chars() {
        let id = generate_random_id();
        assert_eq!(id.len(), 9);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn task_ids_are_different() {
        let a = generate_random_id();
        let b = generate_random_id();
        assert_ne!(a, b);
    }

    #[test]
    fn secure_compare_works() {
        assert!(secure_compare("1234".as_bytes(), "1234".as_bytes()));
        assert!(!secure_compare("1234".as_bytes(), "1235".as_bytes()));
        assert!(!secure_compare("1234".as_bytes(), "12345".as_bytes()));
        assert!(secure_compare(b"", b""));
    }

    #[test]
    fn cookie_header_has_required_attributes() {
        let h = build_session_cookie_header("abc123", 24, false);
        assert!(h.starts_with("TODO_PIN=abc123"));
        assert!(h.contains("Path=/"));
        assert!(h.contains("HttpOnly"));
        assert!(h.contains("SameSite=Strict"));
        assert!(h.contains("Max-Age=86400"));
        assert!(!h.contains("Secure"));
    }

    #[test]
    fn cookie_header_omits_secure_when_not_https() {
        let h = build_session_cookie_header("x", 1, false);
        assert!(!h.contains("; Secure"));
    }

    #[test]
    fn cookie_header_includes_secure_when_https() {
        let h = build_session_cookie_header("x", 1, true);
        assert!(h.contains("; Secure"));
        assert!(h.contains("Max-Age=3600"));
    }
}
