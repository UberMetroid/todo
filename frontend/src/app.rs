use crate::storage::StorageService;
use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::api;
use crate::header::Header;
use crate::i18n;
use crate::login::Login;
use crate::todo_list::TodoList;
use crate::types::ToastType;
use shared::{PinRequiredResponse, SiteConfig, TodoLists};

#[function_component(App)]
pub fn app() -> Html {
    let site_config = use_state(|| None::<SiteConfig>);
    let pin_required = use_state(|| {
        Some(PinRequiredResponse {
            required: true,
            length: 4,
            locked: false,
            attempts_left: 5,
            lockout_minutes: 15,
            enable_translation: false,
        })
    });
    let authenticated = use_state(|| false);
    let todos = use_state(|| None::<TodoLists>);
    let current_list = use_state(|| "List 1".to_string());
    let active_notification = use_state(|| None::<(String, String)>);
    let active_timeout = use_mut_ref(|| None::<Timeout>);
    let pin_error = use_state(|| None::<String>);
    let (theme, toggle_theme) = crate::theme::use_theme();
    let locale = use_state(|| {
        let local_lang = StorageService::get_item("lang", "en");
        i18n::Locale::from_str(&local_lang)
    });

    {
        let locale = locale.clone();
        use_effect_with(locale.clone(), move |loc| {
            StorageService::set_item("lang", loc.to_str());
        });
    }

    let show_toast = {
        let active_notification = active_notification.clone();
        let active_timeout = active_timeout.clone();
        Callback::from(move |(message, toast_type): (String, ToastType)| {
            let cls = match toast_type {
                ToastType::Success => "success",
                ToastType::Error => "error",
            };
            active_notification.set(Some((message, cls.to_string())));
            if let Some(t) = active_timeout.borrow_mut().take() {
                t.cancel();
            }
            let active_notif = active_notification.clone();
            let timer = active_timeout.clone();
            let new_timer = Timeout::new(3000, move || {
                active_notif.set(None);
                *timer.borrow_mut() = None;
            });
            *active_timeout.borrow_mut() = Some(new_timer);
        })
    };

    let load_todos = {
        let (todos, current_list, authenticated, show_toast) = (
            todos.clone(),
            current_list.clone(),
            authenticated.clone(),
            show_toast.clone(),
        );
        move || {
            let (todos, current_list, authenticated, show_toast) = (
                todos.clone(),
                current_list.clone(),
                authenticated.clone(),
                show_toast.clone(),
            );
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_todos_raw().await {
                    Ok(resp) => {
                        if resp.status() == 401 {
                            authenticated.set(false);
                        } else if let Ok(data) = resp.json::<TodoLists>().await {
                            authenticated.set(true);
                            if !data.is_empty() && !data.contains_key(&*current_list) {
                                if let Some(first_key) = data.keys().next() {
                                    current_list.set(first_key.clone());
                                }
                            }
                            todos.set(Some(data));
                        }
                    }
                    Err(_) => {
                        show_toast.emit(("Failed to load todos".to_string(), ToastType::Error))
                    }
                }
            });
        }
    };

    {
        let (site_config, pin_required, load_todos) = (
            site_config.clone(),
            pin_required.clone(),
            load_todos.clone(),
        );
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(config) = api::fetch_config().await {
                    if let Some(win) = web_sys::window() {
                        if let Some(doc) = win.document() {
                            doc.set_title(&config.site_title);
                        }
                    }
                    site_config.set(Some(config));
                }
            });
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(data) = api::fetch_pin_required().await {
                    pin_required.set(Some(data));
                }
            });
            load_todos();
        });
    }



    let verify_submit_pin = {
        let (pin_error, pin_required, load_todos, show_toast) = (
            pin_error.clone(),
            pin_required.clone(),
            load_todos.clone(),
            show_toast.clone(),
        );
        move |pin: String| {
            let (pin_error, pin_required, load_todos, show_toast) = (
                pin_error.clone(),
                pin_required.clone(),
                load_todos.clone(),
                show_toast.clone(),
            );
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(data) = api::verify_pin(&pin).await {
                    if data.valid {
                        pin_error.set(None);
                        load_todos();
                        show_toast
                            .emit(("Authenticated successfully".to_string(), ToastType::Success));
                    } else {
                        pin_error.set(data.error.clone());
                        if let Some(left) = data.attempts_left {
                            let mut updated = (*pin_required).clone().unwrap();
                            updated.attempts_left = left;
                            if let Some(locked) = data.locked {
                                updated.locked = locked;
                            }
                            if let Some(m) = data.lockout_minutes {
                                updated.lockout_minutes = m;
                            }
                            pin_required.set(Some(updated));
                        }
                    }
                }
            });
        }
    };

    let is_auth = *authenticated
        || pin_required
            .as_ref()
            .map(|pr| !pr.required)
            .unwrap_or(false);
    let site_config_fallback = site_config.as_ref().cloned().unwrap_or_else(|| SiteConfig {
        site_title: "RustDo".to_string(),
        single_list: false,
    });
    let is_pin_required = pin_required.as_ref().map(|pr| pr.required).unwrap_or(false);
    let on_logout = {
        let (authenticated, show_toast, todos) =
            (authenticated.clone(), show_toast.clone(), todos.clone());
        Callback::from(move |_| {
            let (authenticated, show_toast, todos) =
                (authenticated.clone(), show_toast.clone(), todos.clone());
            wasm_bindgen_futures::spawn_local(async move {
                if matches!(api::logout().await, Ok(true)) {
                    authenticated.set(false);
                    todos.set(None);
                    show_toast.emit(("Logged out successfully".to_string(), ToastType::Success));
                } else {
                    show_toast.emit(("Failed to log out".to_string(), ToastType::Error));
                }
            });
        })
    };

    let disable_print = todos.as_ref().map(|t| t.values().all(|v| v.is_empty())).unwrap_or(true);

    let enable_translation = pin_required.as_ref().map(|p| p.enable_translation).unwrap_or(false);

    html! {
        <ContextProvider<i18n::I18nContext> context={locale}>
            <Header
                site_config={site_config_fallback}
                theme={(*theme).clone()}
                on_toggle_theme={toggle_theme.clone()}
                is_authenticated={*authenticated}
                is_pin_required={is_pin_required}
                on_logout={on_logout}
                disable_print={disable_print}
                enable_translation={enable_translation}
            />
            <div class="container">
                if is_auth {
                    if let (Some(config), Some(_)) = (site_config.as_ref(), todos.as_ref()) {
                        <TodoList
                            site_config={config.clone()}
                            todos={todos.clone()}
                            current_list={current_list.clone()}
                            theme={(*theme).clone()}
                            on_toggle_theme={toggle_theme.clone()}
                            show_toast={show_toast.clone()}
                        />
                    }
                } else {
                    if let Some(pr) = pin_required.as_ref() {
                        <Login
                            pin_required={pr.clone()}
                            pin_error={(*pin_error).clone()}
                            on_submit={verify_submit_pin}
                            theme={(*theme).clone()}
                            on_toggle_theme={toggle_theme}
                        />
                    }
                }
            </div>
            <footer class="layout-footer">
                {
                    if let Some((msg, cls)) = &*active_notification {
                        html! { <div class={format!("footer-status-text {}", cls)}>{ msg }</div> }
                    } else {
                        html! { <div class="footer-status-text success">{"Ready"}</div> }
                    }
                }
            </footer>
        </ContextProvider<i18n::I18nContext>>
    }
}
