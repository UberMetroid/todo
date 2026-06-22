use crate::i18n::{use_i18n, TransKey};
use shared::PinRequiredResponse;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub pin_required: PinRequiredResponse,
    pub pin_error: Option<String>,
    pub on_submit: Callback<String>,
    pub theme: String,
    pub on_toggle_theme: Callback<MouseEvent>,
}

#[function_component(Login)]
pub fn login(props: &LoginProps) -> Html {
    let pr = &props.pin_required;
    let (locale, set_locale, t) = use_i18n();

    let pin_input = use_state(|| "".to_string());
    let input_ref = use_node_ref();

    {
        let input_ref = input_ref.clone();
        let is_locked = pr.locked;
        use_effect_with(is_locked, move |locked| {
            if !*locked {
                if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                    let _ = input.focus();
                }
            }
        });
    }

    {
        let pin_input = pin_input.clone();
        let attempts_left = pr.attempts_left;
        let pin_error = props.pin_error.clone();
        let input_ref = input_ref.clone();
        use_effect_with((attempts_left, pin_error), move |_| {
            pin_input.set("".to_string());
            if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                input.set_value("");
                let _ = input.focus();
            }
        });
    }

    let on_input = {
        let pin_input = pin_input.clone();
        let pin_len = pr.length;
        let on_submit = props.on_submit.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
            input.set_value(&filtered);

            if filtered.len() <= pin_len {
                pin_input.set(filtered.clone());
                if filtered.len() == pin_len {
                    on_submit.emit(filtered);
                }
            }
        })
    };

    let on_submit = {
        let pin_input = pin_input.clone();
        let pin_len = pr.length;
        let on_submit = props.on_submit.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let val = (*pin_input).clone();
            if val.len() == pin_len {
                on_submit.emit(val);
            }
        })
    };

    let on_toggle_lang = {
        let locale = locale;
        let set_locale = set_locale;
        Callback::from(move |_| {
            set_locale.emit(locale.next());
        })
    };

    let theme_toggle_icon = match props.theme.as_str() {
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
    };

    html! {
        <div class="login-container">
            <button id="lang-toggle" class="lang-toggle" onclick={on_toggle_lang} aria-label="Toggle language">
                { locale.next().to_str().to_uppercase() }
            </button>
            <button id="theme-toggle" class="theme-toggle" onclick={props.on_toggle_theme.clone()} aria-label="Toggle theme">
                {theme_toggle_icon}
            </button>
            <div id="login-content">
                <div class="pin-header">
                    <h1 id="site-title">{"RustDo"}</h1>
                    <h2 id="pin-description">
                        {
                            if pr.locked {
                                t.t(TransKey::LockoutNotice(pr.lockout_minutes as usize))
                            } else {
                                t.t(TransKey::EnterPin)
                            }
                        }
                    </h2>
                </div>
                <form id="pin-form" onsubmit={on_submit}>
                    <div class="pin-wrapper">
                        <input
                            ref={input_ref}
                            type="password"
                            class="pin-input-field"
                            value={(*pin_input).clone()}
                            oninput={on_input}
                            disabled={pr.locked}
                            placeholder={t.t(TransKey::PinInputPlaceholder(pr.length))}
                            maxlength={pr.length.to_string()}
                            autofocus=true
                        />
                    </div>
                </form>
                <div class="pin-status">
                    if pr.locked {
                        <p id="lockoutNotice" class="lockout-notice" style="display: block;">
                            { t.t(TransKey::LockoutNotice(pr.lockout_minutes as usize)) }
                        </p>
                    } else {
                        if pr.attempts_left < 5 {
                            <p id="attemptsRemaining" class="attempts-remaining" style="display: block;">
                                { t.t(TransKey::AttemptsRemaining(pr.attempts_left)) }
                            </p>
                        }
                    }
                    if let Some(ref err) = props.pin_error {
                        <p id="pinError" class="pin-error" style="display: block;">{ err }</p>
                    }
                </div>
            </div>
        </div>
    }
}
