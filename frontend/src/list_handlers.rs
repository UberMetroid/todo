use crate::i18n::{translate, Locale, TransKey};
use crate::types::ToastType;
use shared::TodoLists;
use yew::prelude::*;

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
    locale: Locale,
) -> Callback<()> {
    Callback::from(move |_| {
        let mut data = todos_data.clone();
        let list_name = format!("List {}", data.len() + 1);
        data.insert(list_name.clone(), Vec::new());
        current_list.set(list_name);
        save_list_todos.emit(data);
        show_toast.emit((
            translate(locale, TransKey::NewListAdded),
            ToastType::Success,
        ));
    })
}

pub fn rename_list_handler(
    todos_data: TodoLists,
    current_list: UseStateHandle<String>,
    save_list_todos: Callback<TodoLists>,
    show_toast: Callback<(String, ToastType)>,
    locale: Locale,
) -> Callback<()> {
    Callback::from(move |_| {
        if let Ok(Some(new_name)) = web_sys::window().unwrap().prompt_with_message_and_default(
            &translate(locale, TransKey::PromptRenameList),
            &*current_list,
        ) {
            let clean_name = new_name.trim().to_string();
            if !clean_name.is_empty()
                && clean_name != *current_list
                && !todos_data.contains_key(&clean_name)
            {
                let mut data = todos_data.clone();
                if let Some(items) = data.remove(&*current_list) {
                    data.insert(clean_name.clone(), items);
                    current_list.set(clean_name);
                    save_list_todos.emit(data);
                    show_toast.emit((translate(locale, TransKey::ListRenamed), ToastType::Success));
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
    locale: Locale,
) -> Callback<String> {
    Callback::from(move |list_name: String| {
        if list_name == "List 1" {
            return;
        }
        if todos_data.len() > 1
            && web_sys::window()
                .unwrap()
                .confirm_with_message(&translate(
                    locale,
                    TransKey::ConfirmDeleteList(list_name.clone()),
                ))
                .unwrap_or(false)
        {
            let mut data = todos_data.clone();
            data.remove(&list_name);
            if *current_list == list_name {
                current_list.set(data.keys().next().unwrap().clone());
            }
            save_list_todos.emit(data);
            show_toast.emit((translate(locale, TransKey::ListDeleted), ToastType::Success));
        }
    })
}
