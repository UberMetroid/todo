use crate::state::SharedState;
use axum::{Json, extract::State};
use shared_core::types::SiteConfig;

pub async fn get_config(State(state): State<SharedState>) -> Json<SiteConfig> {
    Json(SiteConfig {
        site_title: state.site_title.clone(),
        single_list: state.single_list,
        enable_themes: state.enable_themes,
        enable_print: state.enable_print,
        show_version: state.show_version,
        show_github: state.show_github,
    })
}
