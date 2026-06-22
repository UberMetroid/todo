use yew::prelude::*;
use web_sys::MouseEvent;
use shared::{SiteConfig, TodoLists};
use crate::types::ToastType;
use crate::api;
use crate::todo_header::TodoHeader;
use crate::todo_form::TodoForm;
use crate::todo_items_list::TodoItemsList;
use crate::todo_list_handlers;

#[derive(Properties, PartialEq)]
pub struct TodoListProps {
    pub site_config: SiteConfig,
    pub todos: UseStateHandle<Option<TodoLists>>,
    pub current_list: UseStateHandle<String>,
    pub theme: String,
    pub on_toggle_theme: Callback<MouseEvent>,
    pub show_toast: Callback<(String, ToastType)>,
}

#[function_component(TodoList)]
pub fn todo_list(props: &TodoListProps) -> Html {
    let custom_select_open = use_state(|| false);
    let editing_todo_id = use_state(|| None::<String>);
    let edit_input_value = use_state(|| String::new());
    let dragged_todo_id = use_state(|| None::<String>);
    let edit_ref = use_node_ref();

    let current_list = &*props.current_list;
    let todos_data = props.todos.as_ref().cloned().unwrap_or_default();
    let current_list_todos = todos_data.get(current_list).cloned().unwrap_or_default();

    let active_count = current_list_todos.iter().filter(|t| !t.completed).count();
    let completed_count = current_list_todos.iter().filter(|t| t.completed).count();

    let mut lists_keys: Vec<String> = todos_data.keys().cloned().collect();
    lists_keys.sort_by(|a, b| {
        if a == "List 1" { return std::cmp::Ordering::Less; }
        if b == "List 1" { return std::cmp::Ordering::Greater; }
        a.cmp(b)
    });

    let save_list_todos = {
        let todos = props.todos.clone();
        let show_toast = props.show_toast.clone();
        Callback::from(move |updated_todos: TodoLists| {
            let todos = todos.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::save_todos(&updated_todos).await {
                    Ok(true) => todos.set(Some(updated_todos)),
                    _ => show_toast.emit(("Failed to save changes".to_string(), ToastType::Error)),
                }
            });
        })
    };

    let on_add_todo = todo_list_handlers::add_todo_handler(
        todos_data.clone(),
        current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_toggle_todo = todo_list_handlers::toggle_todo_handler(
        todos_data.clone(),
        current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_delete_todo = todo_list_handlers::delete_todo_handler(
        todos_data.clone(),
        current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_edit_save_todo = todo_list_handlers::edit_save_todo_handler(
        todos_data.clone(),
        current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_drag_reorder_todo = todo_list_handlers::drag_reorder_todo_handler(
        todos_data.clone(),
        current_list.clone(),
        save_list_todos.clone(),
    );

    let on_clear_completed = todo_list_handlers::clear_completed_handler(
        todos_data.clone(),
        current_list.clone(),
        current_list_todos.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_list_change = todo_list_handlers::list_change_handler(
        props.current_list.clone(),
        custom_select_open.clone(),
    );

    let on_add_list = todo_list_handlers::add_list_handler(
        todos_data.clone(),
        props.current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_rename_list = todo_list_handlers::rename_list_handler(
        todos_data.clone(),
        props.current_list.clone(),
        save_list_todos.clone(),
        props.show_toast.clone(),
    );

    let on_delete_list = todo_list_handlers::delete_list_handler(
        todos_data,
        props.current_list.clone(),
        save_list_todos,
        props.show_toast.clone(),
    );

    html! {
        <div class="app">
            <TodoHeader
                site_config={props.site_config.clone()}
                current_list={current_list.clone()}
                lists_keys={lists_keys}
                custom_select_open={custom_select_open}
                on_list_change={on_list_change}
                on_delete_list={on_delete_list}
                on_rename_list={on_rename_list}
                on_add_list={on_add_list}
                theme={props.theme.clone()}
                on_toggle_theme={props.on_toggle_theme.clone()}
            />
            <main>
                <TodoForm
                    on_add_todo={on_add_todo}
                    on_clear_completed={on_clear_completed}
                />
                <ul id="todoList" class="todo-list">
                    <div class="active-todos">
                        <TodoItemsList
                            current_list_todos={current_list_todos.clone()}
                            on_toggle_todo={on_toggle_todo.clone()}
                            on_delete_todo={on_delete_todo.clone()}
                            on_edit_save_todo={on_edit_save_todo.clone()}
                            dragged_todo_id={dragged_todo_id.clone()}
                            on_drag_reorder_todo={on_drag_reorder_todo.clone()}
                            editing_todo_id={editing_todo_id.clone()}
                            edit_input_value={edit_input_value.clone()}
                            edit_ref={edit_ref.clone()}
                            is_completed={false}
                        />
                    </div>
                    if active_count > 0 && completed_count > 0 {
                        <li class="todo-divider">{"Completed"}</li>
                    }
                    <TodoItemsList
                        current_list_todos={current_list_todos.clone()}
                        on_toggle_todo={on_toggle_todo.clone()}
                        on_delete_todo={on_delete_todo.clone()}
                        on_edit_save_todo={on_edit_save_todo.clone()}
                        dragged_todo_id={dragged_todo_id.clone()}
                        on_drag_reorder_todo={on_drag_reorder_todo.clone()}
                        editing_todo_id={editing_todo_id.clone()}
                        edit_input_value={edit_input_value.clone()}
                        edit_ref={edit_ref.clone()}
                        is_completed={true}
                    />
                </ul>
            </main>
        </div>
    }
}
