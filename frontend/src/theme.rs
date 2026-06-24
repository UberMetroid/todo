use yew::prelude::*;
use crate::storage::StorageService;

#[hook]
pub fn use_theme() -> (UseStateHandle<String>, Callback<MouseEvent>) {
    let theme = use_state(|| {
        let raw = StorageService::get_item("theme", "crateria");
        let theme = match raw.as_str() {
            "light" => "brinstar".to_string(),
            "dark" => "crateria".to_string(),
            "nord" => "maridia".to_string(),
            "dracula" => "wrecked_ship".to_string(),
            "sepia" => "norfair".to_string(),
            t => t.to_string(),
        };
        if theme != raw {
            StorageService::set_item("theme", &theme);
        }
        theme
    });

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new = match theme.as_str() {
                "crateria" => "brinstar",
                "brinstar" => "norfair",
                "norfair" => "wrecked_ship",
                "wrecked_ship" => "maridia",
                "maridia" => "tourian",
                _ => "crateria",
            };
            let el = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .document_element()
                .unwrap();
            let _ = el.set_attribute("data-theme", new);
            let _ = el.set_attribute("class", new);
            StorageService::set_item("theme", new);
            theme.set(new.to_string());
        })
    };

    (theme, toggle_theme)
}
