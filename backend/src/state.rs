use axum::{extract::ConnectInfo, http::HeaderMap};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};
use tokio::sync::RwLock;

pub struct AppState {
    pub pin: Option<String>,
    pub site_title: String,
    pub single_list: bool,
    pub allowed_origins: String,
    pub is_production: bool,
    pub data_file: String,
    pub asset_manifest: Vec<String>,
    pub max_attempts: usize,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    // IP -> (failed_attempts, last_attempt_time)
    pub login_attempts: RwLock<HashMap<String, (usize, Instant)>>,
    pub active_sessions: RwLock<std::collections::HashSet<String>>,
    pub rate_limiter: RwLock<HashMap<String, Vec<Instant>>>,
}

impl AppState {
    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        let max_requests = 100;
        let window = std::time::Duration::from_secs(60);
        let now = std::time::Instant::now();

        let mut map = self.rate_limiter.write().await;
        let timestamps = map.entry(ip.to_string()).or_insert_with(Vec::new);
        
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub async fn clean_old_rate_limits(&self) {
        let window = std::time::Duration::from_secs(60);
        let now = std::time::Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}

pub type SharedState = Arc<AppState>;

// Extracts client IP address taking reverse proxies (e.g. Cloudflare, nginx) into account
pub fn get_client_ip(connect_info: &ConnectInfo<SocketAddr>, headers: &HeaderMap) -> String {
    if let Some(cf_connecting_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip) = cf_connecting_ip.to_str() {
            return ip.to_string();
        }
    }
    if let Some(x_forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_list) = x_forwarded_for.to_str() {
            if let Some(ip) = ip_list.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    if let Some(x_real_ip) = headers.get("x-real-ip") {
        if let Ok(ip) = x_real_ip.to_str() {
            return ip.to_string();
        }
    }
    connect_info.ip().to_string()
}
