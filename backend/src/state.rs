//! Application state shared across handlers.
//!
//! `AppState` holds all per-instance state — environment-derived
//! configuration, in-memory rate-limit data, and the active-session
//! set. PIN-attempt tracking lives **process-globally** inside
//! `shared_backend::auth::attempts` so that lockouts survive handler
//! re-creations (axum builds handlers per request); see
//! [`crate::handlers`] for the call sites.

use axum::http::HeaderMap;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::RwLock;

// `AppState` holds non-`Clone` fields (tokio RwLocks) and is always
// accessed through `Arc<AppState>` (see `SharedState` below). Don't
// add `#[derive(Clone)]` — wrap in `Arc` and clone the Arc instead.
pub struct AppState {
    /// Server-wide PIN, if configured. When `None`, the app is in public
    /// mode and `auth_middleware` short-circuits to allow.
    pub pin: Option<String>,
    pub site_title: String,
    pub single_list: bool,
    pub allowed_origins: String,
    pub is_production: bool,
    pub data_file: String,
    pub asset_manifest: Vec<String>,
    pub max_attempts: usize,
    pub lockout_duration: std::time::Duration,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    pub show_version: bool,
    pub show_github: bool,
    /// Whether to honor `X-Forwarded-For` for client-IP resolution.
    /// When `false`, the connecting socket IP is used (immune to
    /// header-spoofing attacks).
    pub trust_proxy: bool,
    /// Allowlist of proxy CIDRs that are trusted to set `X-Forwarded-For`.
    /// Ignored when `trust_proxy` is `false`.
    pub trusted_proxies: Vec<ipnet::IpNet>,
    /// Cookie lifetime in hours. Passed to the cookie builder.
    pub cookie_max_age_hours: i64,
    /// Active authenticated sessions. `TODO_PIN` cookie value -> present.
    /// We use random session IDs (not the raw PIN) so that a stolen
    /// cookie does not leak the user's PIN.
    pub active_sessions: RwLock<std::collections::HashSet<String>>,
    /// Per-IP sliding-window request timestamps for the custom
    /// `rate_limit_middleware` (in addition to PIN-attempt lockouts).
    pub rate_limiter: RwLock<HashMap<String, Vec<Instant>>>,
}

impl AppState {
    /// Returns `true` if the request from `ip` is under the rate limit
    /// (≤ 100 requests in any 60-second window). Updates the counter.
    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        const MAX_REQUESTS: usize = 100;
        const WINDOW_SECS: u64 = 60;
        let window = std::time::Duration::from_secs(WINDOW_SECS);
        let now = std::time::Instant::now();

        let mut map = self.rate_limiter.write().await;
        let timestamps = map.entry(ip.to_string()).or_insert_with(Vec::new);

        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= MAX_REQUESTS {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    /// Sweep stale entries from the rate-limiter map. Called once a
    /// minute by the background cleanup task in `main.rs`.
    pub async fn clean_old_rate_limits(&self) {
        const WINDOW_SECS: u64 = 60;
        let window = std::time::Duration::from_secs(WINDOW_SECS);
        let now = std::time::Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}

pub type SharedState = Arc<AppState>;

/// Resolve the client IP from the connecting socket and request headers.
///
/// Behaviour matches `shared_backend::server::ip::get_client_ip`:
///
/// 1. If `trust_proxy` is `false` (the safe default), the connecting
///    socket IP is used unconditionally.
/// 2. If `trust_proxy` is `true` and `trusted_proxies` is non-empty, the
///    connecting socket must be inside one of the trusted CIDRs for
///    `X-Forwarded-For` to be honored. This prevents attackers from
///    forging `X-Forwarded-For` from arbitrary origins to rotate their
///    lockout-key IP.
/// 3. If `trust_proxy` is `true` and `trusted_proxies` is empty, the
///    socket IP is used (fail-safe — no header is trusted).
pub fn get_client_ip(
    headers: &HeaderMap,
    socket_addr: SocketAddr,
    trust_proxy: bool,
    trusted_proxies: &[ipnet::IpNet],
) -> String {
    shared_backend::server::get_client_ip(headers, socket_addr, trust_proxy, trusted_proxies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_ip_no_proxy_returns_socket() {
        let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
        let headers = HeaderMap::new();
        let ip = get_client_ip(&headers, socket, false, &[]);
        assert_eq!(ip, "10.0.0.1");
    }

    #[test]
    fn client_ip_proxy_without_list_ignores_xff() {
        let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        // trust_proxy=true with empty allowlist → fail-safe to socket
        let ip = get_client_ip(&headers, socket, true, &[]);
        assert_eq!(ip, "10.0.0.1");
    }

    #[test]
    fn client_ip_proxy_with_allowlist_honors_xff() {
        let socket: SocketAddr = "10.0.0.1:4403".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        let trusted: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
        let ip = get_client_ip(&headers, socket, true, &[trusted]);
        assert_eq!(ip, "1.2.3.4");
    }

    #[tokio::test]
    async fn rate_limit_allows_under_threshold() {
        let state = AppState {
            pin: None,
            site_title: "x".into(),
            single_list: false,
            allowed_origins: "*".into(),
            is_production: false,
            data_file: "x".into(),
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
        };
        for _ in 0..50 {
            assert!(state.check_rate_limit("1.1.1.1").await);
        }
    }

    #[tokio::test]
    async fn rate_limit_blocks_over_threshold() {
        let state = AppState {
            pin: None,
            site_title: "x".into(),
            single_list: false,
            allowed_origins: "*".into(),
            is_production: false,
            data_file: "x".into(),
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
        };
        for _ in 0..100 {
            assert!(state.check_rate_limit("1.1.1.1").await);
        }
        // 101st request should be blocked.
        assert!(!state.check_rate_limit("1.1.1.1").await);
    }
}
