use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn logout(cookie_jar: CookieJar, State(state): State<SharedState>) -> impl IntoResponse {
    if let Some(cookie) = cookie_jar.get("TODO_PIN") {
        state.active_sessions.write().await.remove(cookie.value());
    }
    let cookie = Cookie::build(("TODO_PIN", ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .build();
    (StatusCode::OK, cookie_jar.remove(cookie))
}
