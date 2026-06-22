use yew::prelude::*;
use crate::types::{Toast, ToastType};

#[derive(Properties, PartialEq)]
pub struct ToastListProps {
    pub toasts: Vec<Toast>,
}

// Renders the float toast containers at the bottom of the screen dynamically
#[function_component(ToastList)]
pub fn toast_list(props: &ToastListProps) -> Html {
    html! {
        <div id="toast-container" class="toast-container">
            {
                props.toasts.iter().map(|toast| {
                    let toast_cls = match toast.toast_type {
                        ToastType::Success => "success",
                        ToastType::Error => "error"
                    };
                    html! {
                        <div key={toast.id} class={format!("toast show {}", toast_cls)}>
                            { &toast.message }
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}
