use crate::i18n::{use_i18n, TransKey};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TodoFormProps {
    pub on_add_todo: Callback<SubmitEvent>,
    pub on_clear_completed: Callback<MouseEvent>,
}

// A component representing the form to add new tasks and clear completed ones
#[function_component(TodoForm)]
pub fn todo_form(props: &TodoFormProps) -> Html {
    let on_add = props.on_add_todo.clone();
    let on_clear = props.on_clear_completed.clone();
    let (_, _, t) = use_i18n();

    html! {
        <form id="todoForm" class="todo-form" onsubmit={on_add}>
            <input type="text" name="todoInput" id="todoInput" placeholder={t.t(TransKey::WhatNeedsBeDone)} required=true />
            <div class="button-group">
                <button type="submit">{t.t(TransKey::Add)}</button>
                <button type="button" id="clearCompleted" class="clear-btn" aria-label={t.t(TransKey::DeleteCompleted)} onclick={on_clear}>
                    <svg viewBox="0 0 24 24" width="16" height="16">
                        <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                </button>
            </div>
        </form>
    }
}
