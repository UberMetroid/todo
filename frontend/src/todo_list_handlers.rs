use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlFormElement, MouseEvent};
use wasm_bindgen::JsCast;
use shared::{TodoLists, TodoItem};
use crate::types::ToastType;

pub fn add_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<SubmitEvent> {
    Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let form = e.target_dyn_into::<HtmlFormElement>().unwrap();
        let input_el = form.elements().get_with_name("todoInput").unwrap().dyn_into::<HtmlInputElement>().unwrap();
        let text = input_el.value().trim().to_string();
        if !text.is_empty() {
            let mut data = todos_data.clone();
            let unique_id = format!("{}-{}", js_sys::Date::now(), (js_sys::Math::random() * 1000000.0) as u32);
            data.entry(current_list.clone()).or_insert_with(Vec::new).push(TodoItem { id: unique_id, text, completed: false });
            save_list_todos.emit(data);
            input_el.set_value("");
            show_toast.emit(("Task added".to_string(), ToastType::Success));
        }
    })
}

pub fn toggle_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<String> {
    Callback::from(move |id: String| {
        let mut data = todos_data.clone();
        if let Some(item) = data.get_mut(&current_list).and_then(|l| l.iter_mut().find(|t| t.id == id)) {
            item.completed = !item.completed;
            let msg = if item.completed { "Task completed! 🎉" } else { "Task uncompleted" };
            save_list_todos.emit(data);
            show_toast.emit((msg.to_string(), ToastType::Success));
        }
    })
}

pub fn delete_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<(String, String)> {
    Callback::from(move |(id, text): (String, String)| {
        let window = web_sys::window().unwrap();
        if window.confirm_with_message(&format!("Are you sure you want to delete \"{}\"?", text)).unwrap_or(false) {
            let mut data = todos_data.clone();
            if let Some(list) = data.get_mut(&current_list) {
                list.retain(|t| t.id != id);
                save_list_todos.emit(data);
                show_toast.emit(("Task deleted".to_string(), ToastType::Error));
            }
        }
    })
}

pub fn edit_save_todo_handler(
    todos_data: TodoLists,
    current_list: String,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<(String, String)> {
    Callback::from(move |(id, new_text): (String, String)| {
        let clean_text = new_text.trim().to_string();
        if !clean_text.is_empty() {
            let mut data = todos_data.clone();
            if let Some(item) = data.get_mut(&current_list).and_then(|l| l.iter_mut().find(|t| t.id == id)) {
                if item.text != clean_text {
                    item.text = clean_text;
                    save_list_todos.emit(data);
                    show_toast.emit(("Task updated".to_string(), ToastType::Success));
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
) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        let completed_count = current_list_todos.iter().filter(|t| t.completed).count();
        if completed_count == 0 {
            show_toast.emit(("No completed tasks to clear".to_string(), ToastType::Error));
            return;
        }
        if web_sys::window().unwrap().confirm_with_message(&format!("Delete {} completed task{}?", completed_count, if completed_count == 1 { "" } else { "s" })).unwrap_or(false) {
            let mut data = todos_data.clone();
            data.get_mut(&current_list).unwrap().retain(|t| !t.completed);
            save_list_todos.emit(data);
            show_toast.emit((format!("Cleared {} completed task{}", completed_count, if completed_count == 1 { "" } else { "s" }), ToastType::Success));
        }
    })
}

pub fn list_change_handler(
    current_list: UseStateHandle<String>,
    custom_select_open: UseStateHandle<bool>,
) -> Callback<String> {
    Callback::from(move |list_name: String| {
        current_list.set(list_name);
        custom_select_open.set(false);
    })
}

pub fn add_list_handler(
    todos_data: TodoLists,
    current_list: UseStateHandle<String>,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<()> {
    Callback::from(move |_| {
        let mut data = todos_data.clone();
        let list_name = format!("List {}", data.len() + 1);
        data.insert(list_name.clone(), Vec::new());
        current_list.set(list_name);
        save_list_todos.emit(data);
        show_toast.emit(("New list added".to_string(), ToastType::Success));
    })
}

pub fn rename_list_handler(
    todos_data: TodoLists,
    current_list: UseStateHandle<String>,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<()> {
    Callback::from(move |_| {
        if let Ok(Some(new_name)) = web_sys::window().unwrap().prompt_with_message_and_default("Enter new list name:", &*current_list) {
            let clean_name = new_name.trim().to_string();
            if !clean_name.is_empty() && clean_name != *current_list && !todos_data.contains_key(&clean_name) {
                let mut data = todos_data.clone();
                if let Some(items) = data.remove(&*current_list) {
                    data.insert(clean_name.clone(), items);
                    current_list.set(clean_name);
                    save_list_todos.emit(data);
                    show_toast.emit(("List renamed".to_string(), ToastType::Success));
                }
            }
        }
    })
}

pub fn delete_list_handler(
    todos_data: TodoLists,
    current_list: UseStateHandle<String>,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
) -> Callback<String> {
    Callback::from(move |list_name: String| {
        if list_name == "List 1" { return; }
        if todos_data.len() > 1 && web_sys::window().unwrap().confirm_with_message(&format!("Delete \"{}\" and all its tasks?", list_name)).unwrap_or(false) {
            let mut data = todos_data.clone();
            data.remove(&list_name);
            if *current_list == list_name {
                current_list.set(data.keys().next().unwrap().clone());
            }
            save_list_todos.emit(data);
            show_toast.emit(("List deleted".to_string(), ToastType::Success));
        }
    })
}
