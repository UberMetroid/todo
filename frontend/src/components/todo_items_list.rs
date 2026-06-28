use crate::components::todo_item::TodoItemComponent;
use shared_core::types::TodoItem;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TodoItemsListProps {
    pub current_list_todos: Vec<TodoItem>,
    pub on_toggle_todo: Callback<String>,
    pub on_delete_todo: Callback<(String, String)>,
    pub on_edit_save_todo: Callback<(String, String)>,
    pub dragged_todo_id: UseStateHandle<Option<String>>,
    pub on_drag_reorder_todo: Callback<(String, String)>,
    pub editing_todo_id: UseStateHandle<Option<String>>,
    pub edit_input_value: UseStateHandle<String>,
    pub edit_ref: NodeRef,
    pub is_completed: bool,
}

// Renders the list of active or completed todo items
#[function_component(TodoItemsList)]
pub fn todo_items_list(props: &TodoItemsListProps) -> Html {
    props
        .current_list_todos
        .iter()
        .filter(|t| t.completed == props.is_completed)
        .map(|item| {
            html! {
                <TodoItemComponent
                    item={item.clone()}
                    on_toggle={props.on_toggle_todo.clone()}
                    on_delete={props.on_delete_todo.clone()}
                    on_edit_save={props.on_edit_save_todo.clone()}
                    dragged_todo_id={props.dragged_todo_id.clone()}
                    on_drag_reorder={props.on_drag_reorder_todo.clone()}
                    editing_todo_id={props.editing_todo_id.clone()}
                    edit_input_value={props.edit_input_value.clone()}
                    edit_ref={props.edit_ref.clone()}
                />
            }
        })
        .collect::<Html>()
}
