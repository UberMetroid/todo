use crate::i18n::use_i18n;
use shared::SiteConfig;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub site_config: SiteConfig,
    pub theme: String,
    pub on_toggle_theme: Callback<MouseEvent>,
    pub is_authenticated: bool,
    pub is_pin_required: bool,
    pub on_logout: Callback<MouseEvent>,
}

// Renders the header section of the todo application with header-title, list selector, and header-right controls
#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let on_toggle = props.on_toggle_theme.clone();
    let (locale, set_locale, _) = use_i18n();

    let on_select_lang = {
        let set_locale = set_locale;
        Callback::from(move |e: Event| {
            let select = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let new_loc = crate::i18n::Locale::from_str(&select.value());
            set_locale.emit(new_loc);
        })
    };

    let logout_text = match locale {
        crate::i18n::Locale::Zh => "退出登录",
        crate::i18n::Locale::Es => "Cerrar sesión",
        crate::i18n::Locale::De => "Abmelden",
        crate::i18n::Locale::Ja => "ログアウト",
        crate::i18n::Locale::Fr => "Se déconnecter",
        crate::i18n::Locale::Pt => "Sair",
        crate::i18n::Locale::Ru => "Выйти",
        _ => "Log out",
    };

    let disabled = !props.is_authenticated || !props.is_pin_required;
    let onclick_handler = if disabled {
        Callback::from(|_| ())
    } else {
        props.on_logout.clone()
    };

    html! {
        <header>
            <div id="header-title">
                <h1>{ &props.site_config.site_title }</h1>
            </div>
            <div class="header-right">
                <div class="language-select-container">
                    <select
                        class="language-select"
                        id="language-select"
                        value={locale.to_str()}
                        onchange={on_select_lang}
                        aria-label="Select language"
                    >
                        {
                            for crate::i18n::Locale::all().iter().map(|loc| {
                                html! {
                                    <option value={loc.to_str()} selected={*loc == locale}>
                                        { loc.display_label() }
                                    </option>
                                }
                            })
                        }
                    </select>
                </div>
                <button id="theme-toggle" class="icon-button" aria-label="Toggle theme" onclick={on_toggle}>
                    {
                        match props.theme.as_str() {
                            "dark" => html! {
                                <svg id="moon-icon" class="moon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" /></svg>
                            },
                            "nord" => html! {
                                <svg id="droplet-icon" class="droplet" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22a7 7 0 0 0 7-7c0-4.3-7-13-7-13S5 10.7 5 15a7 7 0 0 0 7 7z"/></svg>
                            },
                            "dracula" => html! {
                                <svg id="sparkles-icon" class="sparkles" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275Z"/><path d="m5 3 1 2.5L8.5 6 6 7 5 9.5 4 7 1.5 6 4 5Z"/><path d="m19 17 1 2.5 2.5.5-2.5 1-1 2.5-1-2.5-2.5-1 2.5-1Z"/></svg>
                            },
                            "sepia" => html! {
                                <svg id="coffee-icon" class="coffee" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 8h1a4 4 0 1 1 0 8h-1"/><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/><line x1="6" y1="2" x2="6" y2="4"/><line x1="10" y1="2" x2="10" y2="4"/><line x1="14" y1="2" x2="14" y2="4"/></svg>
                            },
                            _ => html! {
                                <svg id="sun-icon" class="sun" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="M4.93 4.93l1.41 1.41" /><path d="M17.66 17.66l1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="M6.34 17.66l-1.41 1.41" /><path d="M19.07 4.93l-1.41 1.41" /></svg>
                            },
                        }
                    }
                </button>
                <button
                    id="logout-button"
                    class="icon-button"
                    onclick={onclick_handler}
                    disabled={disabled}
                    data-tooltip={if disabled { "".to_string() } else { logout_text.to_string() }}
                >
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                        <polyline points="16 17 21 12 16 7" />
                        <line x1="21" y1="12" x2="9" y2="12" />
                    </svg>
                </button>
            </div>
        </header>
    }
}
