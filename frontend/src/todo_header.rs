use crate::i18n::use_i18n;
use crate::list_selector::ListSelector;
use shared::SiteConfig;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TodoHeaderProps {
    pub site_config: SiteConfig,
    pub current_list: String,
    pub lists_keys: Vec<String>,
    pub custom_select_open: UseStateHandle<bool>,
    pub on_list_change: Callback<String>,
    pub on_delete_list: Callback<String>,
    pub on_rename_list: Callback<()>,
    pub on_add_list: Callback<()>,
    pub theme: String,
    pub on_toggle_theme: Callback<MouseEvent>,
}

// Renders the header section of the todo application
#[function_component(TodoHeader)]
pub fn todo_header(props: &TodoHeaderProps) -> Html {
    let on_toggle = props.on_toggle_theme.clone();
    let (locale, set_locale, _) = use_i18n();

    let on_toggle_lang = {
        let locale = locale;
        let set_locale = set_locale;
        Callback::from(move |_| {
            set_locale.emit(locale.next());
        })
    };

    html! {
        <header>
            <h1 id="header-title">{ &props.site_config.site_title }</h1>
            if !props.site_config.single_list {
                <ListSelector
                    current_list={props.current_list.clone()}
                    lists_keys={props.lists_keys.clone()}
                    custom_select_open={props.custom_select_open.clone()}
                    on_list_change={props.on_list_change.clone()}
                    on_delete_list={props.on_delete_list.clone()}
                    on_rename_list={props.on_rename_list.clone()}
                    on_add_list={props.on_add_list.clone()}
                />
            }
            <div class="header-controls">
                <button id="langToggle" class="lang-toggle-header" onclick={on_toggle_lang} aria-label="Toggle language">
                    { locale.next().to_str().to_uppercase() }
                </button>
                <button id="themeToggle" aria-label="Toggle theme" onclick={on_toggle}>
                    {
                        match props.theme.as_str() {
                            "dark" => html! {
                                <svg id="moon-icon" class="moon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" /></svg>
                            },
                            "nord" => html! {
                                <svg id="droplet-icon" class="droplet" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22a7 7 0 0 0 7-7c0-4.3-7-13-7-13S5 10.7 5 15a7 7 0 0 0 7 7z"/></svg>
                            },
                            "dracula" => html! {
                                <svg id="sparkles-icon" class="sparkles" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275Z"/><path d="m5 3 1 2.5L8.5 6 6 7 5 9.5 4 7 1.5 6 4 5Z"/><path d="m19 17 1 2.5 2.5.5-2.5 1-1 2.5-1-2.5-2.5-1 2.5-1Z"/></svg>
                            },
                            "sepia" => html! {
                                <svg id="coffee-icon" class="coffee" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 8h1a4 4 0 1 1 0 8h-1"/><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/><line x1="6" y1="2" x2="6" y2="4"/><line x1="10" y1="2" x2="10" y2="4"/><line x1="14" y1="2" x2="14" y2="4"/></svg>
                            },
                            _ => html! {
                                <svg id="sun-icon" class="sun" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="M4.93 4.93l1.41 1.41" /><path d="M17.66 17.66l1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="M6.34 17.66l-1.41 1.41" /><path d="M19.07 4.93l-1.41 1.41" /></svg>
                            },
                        }
                    }
                </button>
            </div>
        </header>
    }
}
