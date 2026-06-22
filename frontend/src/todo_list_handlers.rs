use crate::i18n::{translate, Locale, TransKey};
use crate::types::ToastType;
use shared::{TodoItem, TodoLists};
use wasm_bindgen::JsCast;
use web_sys::{HtmlFormElement, HtmlInputElement, MouseEvent};
use yew::prelude::*;

pub fn add_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<SubmitEvent> {
    Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = e.target_dyn_into::<HtmlFormElement>().unwrap();
        let input_el = form
            .elements()
            .get_with_name("todoInput")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let text = input_el.value().trim().to_string();
        if !text.is_empty() {
            let mut data = todos_data.clone();
            let unique_id = format!(
                "{}-{}",
                js_sys::Date::now(),
                (js_sys::Math::random() * 1000000.0) as u32
            );
            data.entry(current_list.clone())
                .or_default()
                .push(TodoItem {
                    id: unique_id,
                    text,
                    completed: false,
                });
            save_list_todos.emit(data);
            input_el.set_value("");
            show_toast.emit((translate(locale, TransKey::TaskAdded), ToastType::Success));
        }
    })
}

pub fn toggle_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<String> {
    Callback::from(move |id: String| {
        let mut data = todos_data.clone();
        if let Some(item) = data
            .get_mut(&current_list)
            .and_then(|l| l.iter_mut().find(|t| t.id == id))
        {
            item.completed = !item.completed;
            let msg = if item.completed {
                translate(locale, TransKey::TaskCompleted)
            } else {
                translate(locale, TransKey::TaskUncompleted)
            };
            save_list_todos.emit(data);
            show_toast.emit((msg, ToastType::Success));
        }
    })
}

pub fn delete_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<(String, String)> {
    Callback::from(move |(id, text): (String, String)| {
        let window = web_sys::window().unwrap();
        if window
            .confirm_with_message(&translate(locale, TransKey::ConfirmDeleteTask(text)))
            .unwrap_or(false)
        {
            let mut data = todos_data.clone();
            if let Some(list) = data.get_mut(&current_list) {
                list.retain(|t| t.id != id);
                save_list_todos.emit(data);
                show_toast.emit((translate(locale, TransKey::TaskDeleted), ToastType::Error));
            }
        }
    })
}

pub fn edit_save_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<(String, String)> {
    Callback::from(move |(id, new_text): (String, String)| {
        let clean_text = new_text.trim().to_string();
        if !clean_text.is_empty() {
            let mut data = todos_data.clone();
            if let Some(item) = data
                .get_mut(&current_list)
                .and_then(|l| l.iter_mut().find(|t| t.id == id))
            {
                if item.text != clean_text {
                    item.text = clean_text;
                    save_list_todos.emit(data);
                    show_toast.emit((translate(locale, TransKey::TaskUpdated), ToastType::Success));
                }
            }
        }
    })
}

pub fn drag_reorder_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
) -> Callback<(String, String)> {
    Callback::from(move |(drag_id, target_id): (String, String)| {
        let mut data = todos_data.clone();
        if let Some(list) = data.get_mut(&current_list) {
            let drag_idx = list.iter().position(|t| t.id == drag_id);
            let target_idx = list.iter().position(|t| t.id == target_id);
            if let (Some(di), Some(ti)) = (drag_idx, target_idx) {
                let item = list.remove(di);
                list.insert(ti, item);
                save_list_todos.emit(data);
            }
        }
    })
}

pub fn clear_completed_handler(
    todos_data: TodoLists,
    current_list: String,
    current_list_todos: Vec<TodoItem>,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let completed_count = current_list_todos.iter().filter(|t| t.completed).count();
        if completed_count == 0 {
            show_toast.emit((
                translate(locale, TransKey::NoCompletedTasks),
                ToastType::Error,
            ));
            return;
        }
        if web_sys::window()
            .unwrap()
            .confirm_with_message(&translate(
                locale,
                TransKey::ConfirmDeleteCompleted(completed_count),
            ))
            .unwrap_or(false)
        {
            let mut data = todos_data.clone();
            data.get_mut(&current_list)
                .unwrap()
                .retain(|t| !t.completed);
            save_list_todos.emit(data);
            show_toast.emit((
                translate(locale, TransKey::ClearedCompleted(completed_count)),
                ToastType::Success,
            ));
        }
    })
}
