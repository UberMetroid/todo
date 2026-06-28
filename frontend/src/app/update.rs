use crate::api;
use crate::app::{App, Msg};
use crate::storage::StorageService;
use crate::types::ToastType;
use shared::PinRequiredResponse;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use shared_assets::theme::{Theme, mapping::Scheme};

impl App {
    pub fn create_app(ctx: &Context<Self>) -> Self {
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
        
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
            && let Some(el) = doc.document_element()
        {
            let _ = el.set_attribute("data-theme", &theme);
            let _ = el.set_attribute("class", &theme);
        }


        let local_lang = StorageService::get_item("lang", "en");
        let locale = crate::i18n::Locale::from_str(&local_lang);

        let link = ctx.link().clone();
        spawn_local(async move {
            if let Ok(config) = api::fetch_config().await {
                link.send_message(Msg::LoadConfig(config));
            }
        });

        let link = ctx.link().clone();
        spawn_local(async move {
            if let Ok(data) = api::fetch_pin_required().await {
                link.send_message(Msg::LoadPinRequired(data));
            }
        });

        let link = ctx.link().clone();
        spawn_local(async move {
            match api::fetch_todos_raw().await {
                Ok(resp) => {
                    if resp.status() == 401 {
                        link.send_message(Msg::SetAuthenticated(false));
                    } else if let Ok(data) = resp.json::<shared::TodoLists>().await {
                        link.send_message(Msg::LoadTodosSuccess(data));
                    }
                }
                Err(_) => {
                    link.send_message(Msg::LoadTodosError);
                }
            }
        });

        Self {
            site_config: None,
            pin_required: Some(PinRequiredResponse {
                required: true,
                length: 4,
                locked: false,
                attempts_left: 5,
                lockout_minutes: 15,
                enable_translation: false,
                enable_themes: true,
                enable_print: false,
                show_version: true,
                show_github: true,
            }),
            authenticated: false,
            todos: None,
            current_list: "List 1".to_string(),
            active_notification: None,
            active_timeout: None,
            pin_error: None,
            theme,
            locale,
        }
    }

    pub fn load_todos_fn(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        spawn_local(async move {
            match api::fetch_todos_raw().await {
                Ok(resp) => {
                    if resp.status() == 401 {
                        link.send_message(Msg::SetAuthenticated(false));
                    } else if let Ok(data) = resp.json::<shared::TodoLists>().await {
                        link.send_message(Msg::LoadTodosSuccess(data));
                    }
                }
                Err(_) => {
                    link.send_message(Msg::LoadTodosError);
                }
            }
        });
    }

    pub fn update_app(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::LoadConfig(config) => {
                if let Some(win) = web_sys::window()
                    && let Some(doc) = win.document()
                {
                    doc.set_title(&config.site_title);
                }
                if !config.enable_themes {
                    self.theme = "tourian".to_string();
                    StorageService::set_item("theme", "tourian");
                    if let Some(win) = web_sys::window()
                        && let Some(doc) = win.document()
                        && let Some(el) = doc.document_element()
                    {
                        let _ = el.set_attribute("data-theme", "tourian");
                        let _ = el.set_attribute("class", "tourian");
                    }
                }
                self.site_config = Some(config);
                true
            }
            Msg::LoadPinRequired(pr) => {
                self.pin_required = Some(pr);
                true
            }
            Msg::SetAuthenticated(auth) => {
                self.authenticated = auth;
                true
            }
            Msg::LoadTodosSuccess(data) => {
                self.authenticated = true;
                if !data.is_empty()
                    && !data.contains_key(&self.current_list)
                    && let Some(first_key) = data.keys().next()
                {
                    self.current_list = first_key.clone();
                }
                self.todos = Some(data);
                true
            }
            Msg::LoadTodosError => {
                ctx.link().send_message(Msg::ShowToast("Failed to load todos".to_string(), ToastType::Error));
                true
            }
            Msg::SwitchLanguage(loc) => {
                StorageService::set_item("lang", loc.to_str());
                self.locale = loc;
                true
            }
            Msg::ToggleTheme => {
                let current = Theme::from_name(&self.theme).unwrap_or_default();
                let next = match current {
                    Theme::Brinstar => Theme::Norfair,
                    Theme::Norfair => Theme::WreckedShip,
                    Theme::WreckedShip => Theme::Maridia,
                    Theme::Maridia => Theme::Tourian,
                    Theme::Tourian => Theme::Crateria,
                    Theme::Crateria => Theme::Brinstar,
                };
                StorageService::set_item("theme", next.name());
                if let Some(html) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.document_element())
                {
                    let _ = html.set_attribute("data-theme", next.name());
                    let _ = html.set_attribute("class", next.name());
                }
                self.theme = next.name().to_string();
                true
            }

            Msg::Logout => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    if matches!(api::logout().await, Ok(true)) {
                        link.send_message(Msg::SetAuthenticated(false));
                        link.send_message(Msg::LoadTodosSuccess(shared::TodoLists::new()));
                        link.send_message(Msg::ShowToast("Logged out successfully".to_string(), ToastType::Success));
                    } else {
                        link.send_message(Msg::ShowToast("Failed to log out".to_string(), ToastType::Error));
                    }
                });
                false
            }
            Msg::SetStatus(status) => {
                self.active_notification = status;
                true
            }
            Msg::VerifyPin(pin) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    if let Ok(data) = api::verify_pin(&pin).await {
                        link.send_message(Msg::PinVerificationResult(data));
                    }
                });
                false
            }
            Msg::PinVerificationResult(data) => {
                if data.valid {
                    self.pin_error = None;
                    self.load_todos_fn(ctx);
                    ctx.link().send_message(Msg::ShowToast("Authenticated successfully".to_string(), ToastType::Success));
                } else {
                    self.pin_error = data.error.clone();
                    if let Some(left) = data.attempts_left
                        && let Some(ref mut pr) = self.pin_required
                    {
                        pr.attempts_left = left;
                        if let Some(locked) = data.locked {
                            pr.locked = locked;
                        }
                        if let Some(m) = data.lockout_minutes {
                            pr.lockout_minutes = m;
                        }
                    }
                }
                true
            }
            Msg::ShowToast(message, toast_type) => {
                let cls = match toast_type {
                    ToastType::Success => "success",
                    ToastType::Error => "error",
                };
                self.active_notification = Some((message, cls.to_string()));
                if let Some(t) = self.active_timeout.take() {
                    t.cancel();
                }
                let link = ctx.link().clone();
                let new_timer = gloo_timers::callback::Timeout::new(3000, move || {
                    link.send_message(Msg::SetStatus(None));
                });
                self.active_timeout = Some(new_timer);
                true
            }
        }
    }
}
