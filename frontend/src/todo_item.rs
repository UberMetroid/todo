use shared::TodoItem as SharedTodoItem;
use web_sys::{DragEvent, HtmlInputElement, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TodoItemProps {
    pub item: SharedTodoItem,
    pub on_toggle: Callback<String>,
    pub on_delete: Callback<(String, String)>,
    pub on_edit_save: Callback<(String, String)>,
    pub dragged_todo_id: UseStateHandle<Option<String>>,
    pub on_drag_reorder: Callback<(String, String)>,
    pub editing_todo_id: UseStateHandle<Option<String>>,
    pub edit_input_value: UseStateHandle<String>,
    pub edit_ref: NodeRef,
}

// Renders an active or completed todo item with drag support, toggle check, and double-click edits
#[function_component(TodoItemComponent)]
pub fn todo_item_component(props: &TodoItemProps) -> Html {
    let item = &props.item;
    let id = item.id.clone();
    let text = item.text.clone();
    let completed = item.completed;

    let ondragstart = {
        let dragged_todo_id = props.dragged_todo_id.clone();
        let id = id.clone();
        Callback::from(move |_e: DragEvent| {
            dragged_todo_id.set(Some(id.clone()));
        })
    };

    let ondragend = {
        let dragged_todo_id = props.dragged_todo_id.clone();
        Callback::from(move |_e: DragEvent| {
            dragged_todo_id.set(None);
        })
    };

    let ondragover = Callback::from(|e: DragEvent| {
        e.prevent_default();
    });

    let ondrop = {
        let dragged_todo_id = props.dragged_todo_id.clone();
        let on_drag_reorder = props.on_drag_reorder.clone();
        let target_id = id.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            if let Some(ref drag_id) = *dragged_todo_id {
                if *drag_id != target_id {
                    on_drag_reorder.emit((drag_id.clone(), target_id.clone()));
                }
            }
        })
    };

    let toggle = {
        let on_toggle = props.on_toggle.clone();
        let id = id.clone();
        move |_| on_toggle.emit(id.clone())
    };

    let delete = {
        let on_delete = props.on_delete.clone();
        let id = id.clone();
        let text = text.clone();
        move |e: MouseEvent| {
            e.stop_propagation();
            on_delete.emit((id.clone(), text.clone()));
        }
    };

    let start_editing = {
        let editing_todo_id = props.editing_todo_id.clone();
        let edit_input_value = props.edit_input_value.clone();
        let id = id.clone();
        let text = text.clone();
        move |_| {
            editing_todo_id.set(Some(id.clone()));
            edit_input_value.set(text.clone());
        }
    };

    html! {
        <li
            class={format!("todo-item {}", if completed { "completed" } else { "" })}
            draggable={(!completed).to_string()}
            {ondragstart}
            {ondragend}
            {ondragover}
            {ondrop}
        >
            <div class="checkbox-wrapper" onclick={toggle.clone()}>
                <input
                    type="checkbox"
                    checked={completed}
                    onchange={move |_| {}}
                />
            </div>

            {
                if Some(id.clone()) == *props.editing_todo_id {
                    let onblur = {
                        let on_edit_save = props.on_edit_save.clone();
                        let id = id.clone();
                        let edit_input_value = props.edit_input_value.clone();
                        let editing_todo_id = props.editing_todo_id.clone();
                        move |_| {
                            on_edit_save.emit((id.clone(), (*edit_input_value).clone()));
                            editing_todo_id.set(None);
                        }
                    };
                    let oninput = {
                        let edit_input_value = props.edit_input_value.clone();
                        move |e: InputEvent| {
                            let el = e.target_dyn_into::<HtmlInputElement>().unwrap();
                            edit_input_value.set(el.value());
                        }
                    };
                    let onkeydown = {
                        let on_edit_save = props.on_edit_save.clone();
                        let id = id.clone();
                        let edit_input_value = props.edit_input_value.clone();
                        let editing_todo_id = props.editing_todo_id.clone();
                        move |e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                on_edit_save.emit((id.clone(), (*edit_input_value).clone()));
                                editing_todo_id.set(None);
                            } else if e.key() == "Escape" {
                                editing_todo_id.set(None);
                            }
                        }
                    };
                    html! {
                        <input
                            ref={props.edit_ref.clone()}
                            type="text"
                            class="edit-input"
                            value={(*props.edit_input_value).clone()}
                            {oninput}
                            {onblur}
                            {onkeydown}
                        />
                    }
                } else {
                    html! {
                        <span onclick={start_editing}>
                            { linkify_text(&text) }
                        </span>
                    }
                }
            }

            <button class="delete-btn" aria-label="Delete todo" onclick={delete}>
                {"×"}
            </button>
        </li>
    }
}

// Convert absolute URLs inside task text to clickable standard links dynamically
fn linkify_text(text: &str) -> Html {
    let mut elements = Vec::new();
    let mut current_text = String::new();

    for word in text.split(' ') {
        if word.starts_with("http://") || word.starts_with("https://") {
            if !current_text.is_empty() {
                elements.push(html! { {&current_text} });
                current_text.clear();
            }
            elements.push(html! {
                <a href={word.to_string()} target="_blank" rel="noopener noreferrer">{word}</a>
            });
            elements.push(html! { " " });
        } else {
            current_text.push_str(word);
            current_text.push(' ');
        }
    }

    if !current_text.is_empty() {
        current_text.pop();
        elements.push(html! { {current_text} });
    }

    html! { { for elements } }
}
