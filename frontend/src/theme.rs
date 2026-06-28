use crate::storage::StorageService;
use yew::prelude::*;
use shared_assets::theme::{Theme, mapping::Scheme};

#[hook]
pub fn use_theme() -> (UseStateHandle<String>, Callback<MouseEvent>) {
    let theme = use_state(|| {
        let raw = StorageService::get_item("theme", Theme::default().name());
        let theme = if let Some(scheme) = Scheme::from_id(&raw) {
            scheme.to_theme().name().to_string()
        } else {
            Theme::from_name(&raw)
                .unwrap_or_default()
                .name()
                .to_string()
        };
        if theme != raw {
            StorageService::set_item("theme", &theme);
        }
        theme
    });

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let current = Theme::from_name(&theme).unwrap_or_default();
            let next = match current {
                Theme::Brinstar => Theme::Norfair,
                Theme::Norfair => Theme::WreckedShip,
                Theme::WreckedShip => Theme::Maridia,
                Theme::Maridia => Theme::Tourian,
                Theme::Tourian => Theme::Crateria,
                Theme::Crateria => Theme::Brinstar,
            };
            let el = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .document_element()
                .unwrap();
            let _ = el.set_attribute("data-theme", next.name());
            let _ = el.set_attribute("class", next.name());
            StorageService::set_item("theme", next.name());
            theme.set(next.name().to_string());
        })
    };

    (theme, toggle_theme)
}

