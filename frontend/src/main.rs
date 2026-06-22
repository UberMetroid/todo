use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
use yew::prelude::*;

mod api;
mod i18n;
mod list_handlers;
mod list_selector;
mod login;
mod toast;
mod todo_form;
mod todo_header;
mod todo_item;
mod todo_items_list;
mod todo_list;
mod todo_list_handlers;
mod types;

use login::Login;
use shared::{PinRequiredResponse, SiteConfig, TodoLists};
use toast::ToastList;
use todo_list::TodoList;
use types::{Toast, ToastType};

#[function_component(App)]
fn app() -> Html {
    let site_config = use_state(|| None::<SiteConfig>);
    let pin_required = use_state(|| None::<PinRequiredResponse>);
    let authenticated = use_state(|| false);
    let todos = use_state(|| None::<TodoLists>);
    let current_list = use_state(|| "List 1".to_string());

    // Toast alerts states
    let toasts = use_state(|| Vec::<Toast>::new());
    let next_toast_id = use_state(|| 0);

    // PIN states
    let pin_error = use_state(|| None::<String>);
    let theme = use_state(|| "light".to_string());
    let locale = use_state(|| {
        let local_lang: String = LocalStorage::get("lang").unwrap_or_else(|_| "en".to_string());
        i18n::Locale::from_str(&local_lang)
    });

    {
        let locale = locale.clone();
        use_effect_with(locale.clone(), move |loc| {
            let _ = LocalStorage::set("lang", loc.to_str());
        });
    }

    let show_toast = {
        let toasts = toasts.clone();
        let next_toast_id = next_toast_id.clone();
        Callback::from(move |(message, toast_type): (String, ToastType)| {
            let id = *next_toast_id;
            next_toast_id.set(id + 1);

            let mut list = (*toasts).clone();
            list.push(Toast::new(id, message, toast_type));
            toasts.set(list);

            let toasts_inner = toasts.clone();
            Timeout::new(3000, move || {
                let mut list = (*toasts_inner).clone();
                list.retain(|t| t.id != id);
                toasts_inner.set(list);
            })
            .forget();
        })
    };

    let load_todos = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let authenticated = authenticated.clone();
        let show_toast = show_toast.clone();
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
                        show_toast.emit(("Failed to load todos".to_string(), ToastType::Error));
                    }
                }
            });
        }
    };

    // Run startup processes: fetch configurations, check themes
    {
        let site_config = site_config.clone();
        let pin_required = pin_required.clone();
        let load_todos = load_todos.clone();
        let theme = theme.clone();
        use_effect_with((), move |_| {
            let local_theme: String =
                LocalStorage::get("theme").unwrap_or_else(|_| "light".to_string());
            let document = web_sys::window().unwrap().document().unwrap();
            let element = document.document_element().unwrap();
            let _ = element.set_attribute("data-theme", &local_theme);
            theme.set(local_theme);

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(config) = api::fetch_config().await {
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

    let toggle_theme = {
        let theme = theme.clone();
        move |_| {
            let new = match theme.as_str() {
                "light" => "dark",
                "dark" => "nord",
                "nord" => "dracula",
                "dracula" => "sepia",
                _ => "light",
            };
            let el = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .document_element()
                .unwrap();
            let _ = el.set_attribute("data-theme", new);
            let _ = LocalStorage::set("theme", new);
            theme.set(new.to_string());
        }
    };

    let verify_submit_pin = {
        let pin_error = pin_error.clone();
        let pin_required = pin_required.clone();
        let load_todos = load_todos.clone();
        let show_toast = show_toast.clone();
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
                        show_toast.emit((
                            "Authenticated successfully 🎉".to_string(),
                            ToastType::Success,
                        ));
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

    let is_auth = *authenticated || pin_required.as_ref().map(|pr| !pr.required).unwrap_or(true);

    html! {
        <ContextProvider<i18n::I18nContext> context={locale}>
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
            <ToastList toasts={(*toasts).clone()} />
        </ContextProvider<i18n::I18nContext>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
