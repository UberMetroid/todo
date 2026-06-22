use crate::i18n::{
    use_i18n,
    TransKey,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TodoFormProps {
    /// Callback triggered when a new todo is submitted
    pub on_add_todo: Callback<SubmitEvent>,
}

/// A component representing the form to add new tasks.
/// Renders a text input field and a submit button.
#[function_component(TodoForm)]
pub fn todo_form(props: &TodoFormProps) -> Html {
    let on_add = props.on_add_todo.clone();
    let (_, _, t) = use_i18n();

    html! {
        <form id="todoForm" class="todo-form" onsubmit={on_add}>
            <input
                type="text"
                name="todoInput"
                id="todoInput"
                placeholder={t.t(TransKey::WhatNeedsBeDone)}
                required=true
            />
            <button type="submit" class="add-todo-btn">
                { t.t(TransKey::Add) }
            </button>
        </form>
    }
}
