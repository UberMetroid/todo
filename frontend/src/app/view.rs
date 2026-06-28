use crate::components::header::Header;
use crate::components::pin::Login;
use crate::components::todo_list::TodoList;
use crate::types::ToastType;
use shared_core::i18n::Language;
use shared_core::types::{PinRequiredResponse, SiteConfig, TodoLists};
use yew::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn render_app(
    locale: UseStateHandle<crate::i18n::Locale>,
    theme: String,
    toggle_theme: Callback<web_sys::MouseEvent>,
    site_config: Option<SiteConfig>,
    pin_required: Option<PinRequiredResponse>,
    authenticated: UseStateHandle<bool>,
    todos: UseStateHandle<Option<TodoLists>>,
    current_list: UseStateHandle<String>,
    active_notification: Option<(String, String)>,
    pin_error: Option<String>,
    verify_submit_pin: Callback<String>,
    on_logout: Callback<web_sys::MouseEvent>,
    show_toast: Callback<(String, ToastType)>,
) -> Html {
    let show_version = site_config
        .as_ref()
        .map(|c| c.show_version)
        .or_else(|| pin_required.as_ref().map(|p| p.show_version))
        .unwrap_or(true);
    let show_github = site_config
        .as_ref()
        .map(|c| c.show_github)
        .or_else(|| pin_required.as_ref().map(|p| p.show_github))
        .unwrap_or(true);
    let version = env!("CARGO_PKG_VERSION").to_string();
    let version_url = format!(
        "https://github.com/UberMetroid/todo/releases/tag/v{}",
        version
    );

    let is_auth = *authenticated
        || pin_required
            .as_ref()
            .map(|pr| !pr.required)
            .unwrap_or(false);

    let site_config_fallback = site_config.as_ref().cloned().unwrap_or_else(|| SiteConfig {
        site_title: "Todo".to_string(),
        single_list: false,
        enable_themes: true,
        enable_print: false,
        show_version: true,
        show_github: true,
    });

    let is_pin_required = pin_required.as_ref().map(|pr| pr.required).unwrap_or(false);

    let disable_print = todos
        .as_ref()
        .map(|t| t.values().all(|v| v.is_empty()))
        .unwrap_or(true);

    let enable_translation = pin_required
        .as_ref()
        .map(|p| p.enable_translation)
        .unwrap_or(false);

    html! {
        <ContextProvider<crate::i18n::I18nContext> context={locale.clone()}>
            <Header
                site_title={site_config_fallback.site_title.clone()}
                theme={theme.clone()}
                language={Language::from_code(locale.to_str())}
                toggle_theme={toggle_theme.clone()}
                on_language_change={
                    let locale = locale.clone();
                    Callback::from(move |lang: Language| {
                        locale.set(crate::i18n::Locale::from_str(lang.code()));
                    })
                }
                is_authenticated={*authenticated}
                pin_required={is_pin_required}
                on_logout={on_logout}
                print_disabled={disable_print}
                enable_translation={enable_translation}
                enable_themes={site_config_fallback.enable_themes}
                on_print={None}
            />
            <div class="container">
                if is_auth {
                    if let (Some(config), Some(_)) = (site_config.as_ref(), todos.as_ref()) {
                        <TodoList
                            site_config={config.clone()}
                            todos={todos.clone()}
                            current_list={current_list.clone()}
                            theme={theme}
                            on_toggle_theme={toggle_theme.clone()}
                            show_toast={show_toast.clone()}
                        />
                    }
                } else {
                    if let Some(pr) = pin_required.as_ref() {
                        <Login
                            pin_required={pr.clone()}
                            pin_error={pin_error.clone()}
                            on_submit={verify_submit_pin}
                            theme={theme}
                            on_toggle_theme={toggle_theme}
                        />
                    }
                }
            </div>
            <crate::components::footer::Footer {show_version} {version} {show_github} {version_url}>
                {
                    if let Some((msg, cls)) = &active_notification {
                        html! { <div class={format!("footer-status-text {}", cls)}>{ msg }</div> }
                    } else {
                        html! { <div class="footer-status-text success">{"Ready"}</div> }
                    }
                }
            </crate::components::footer::Footer>
        </ContextProvider<crate::i18n::I18nContext>>
    }
}
