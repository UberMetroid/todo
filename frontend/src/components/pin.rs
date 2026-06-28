use crate::i18n::{TransKey, use_i18n};
use shared_core::types::PinRequiredResponse;
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
    let (_, _, t) = use_i18n();

    let pin_input = use_state(|| "".to_string());
    let input_ref = use_node_ref();

    {
        let input_ref = input_ref.clone();
        let is_locked = pr.locked;
        use_effect_with(is_locked, move |locked| {
            if !*locked && let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                let _ = input.focus();
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

    html! {
        <div class="login-container">
            <div class="login-box">
                <div class="pin-header">
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
                            <p id="pin-error" class="pin-error" style="display: block;">{ err }</p>
                        }
                </div>
            </div>
        </div>
    }
}
