use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, KeyboardEvent, DragEvent, HtmlFormElement};
use yew::prelude::*;

use shared::{
    PinRequiredResponse, SiteConfig, TodoItem, TodoLists, VerifyPinRequest, VerifyPinResponse,
};

#[derive(Clone, Debug, PartialEq)]
struct Toast {
    id: usize,
    message: String,
    toast_type: ToastType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ToastType {
    Success,
    Error,
}

#[function_component(App)]
fn app() -> Html {
    // App configuration and auth state
    let site_config = use_state(|| None::<SiteConfig>);
    let pin_required = use_state(|| None::<PinRequiredResponse>);
    let authenticated = use_state(|| false);
    
    // Todos state
    let todos = use_state(|| None::<TodoLists>);
    let current_list = use_state(|| "List 1".to_string());
    
    // UI state
    let toasts = use_state(|| Vec::<Toast>::new());
    let next_toast_id = use_state(|| 0);
    let pin_digits = use_state(|| Vec::<String>::new());
    let pin_error = use_state(|| None::<String>);
    let theme = use_state(|| "light".to_string());
    let custom_select_open = use_state(|| false);
    let editing_todo_id = use_state(|| None::<String>);
    let edit_input_value = use_state(|| String::new());
    
    // Drag and Drop state
    let dragged_todo_id = use_state(|| None::<String>);

    // NodeRefs for PIN inputs stored in a State handle to avoid RefCell borrow errors
    let pin_refs = use_state(|| Vec::<NodeRef>::new());
    
    // NodeRef for inline editing input
    let edit_ref = use_node_ref();

    // Toast manager helper
    let show_toast = {
        let toasts = toasts.clone();
        let next_toast_id = next_toast_id.clone();
        move |message: String, toast_type: ToastType| {
            let id = *next_toast_id;
            next_toast_id.set(id + 1);
            
            let mut list = (*toasts).clone();
            list.push(Toast { id, message, toast_type });
            toasts.set(list);

            // Set timeout to remove the toast
            let toasts_inner = toasts.clone();
            Timeout::new(3000, move || {
                let mut list = (*toasts_inner).clone();
                list.retain(|t| t.id != id);
                toasts_inner.set(list);
            })
            .forget();
        }
    };

    // Initialize: Theme
    {
        let theme = theme.clone();
        use_effect_with((), move |_| {
            let local_theme: String = LocalStorage::get("theme")
                .unwrap_or_else(|_| {
                    // Detect system preferences
                    let window = web_sys::window().unwrap();
                    if let Ok(Some(media)) = window.match_media("(prefers-color-scheme: dark)") {
                        if media.matches() {
                            "dark".to_string()
                        } else {
                            "light".to_string()
                        }
                    } else {
                        "light".to_string()
                    }
                });
            
            let document = web_sys::window().unwrap().document().unwrap();
            let element = document.document_element().unwrap();
            let _ = element.set_attribute("data-theme", &local_theme);
            theme.set(local_theme);
        });
    }

    // Toggle theme
    let toggle_theme = {
        let theme = theme.clone();
        move |_| {
            let new_theme = if *theme == "dark" { "light" } else { "dark" };
            let document = web_sys::window().unwrap().document().unwrap();
            let element = document.document_element().unwrap();
            let _ = element.set_attribute("data-theme", new_theme);
            let _ = LocalStorage::set("theme", new_theme);
            theme.set(new_theme.to_string());
        }
    };

    // Load API configs
    let load_todos = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let authenticated = authenticated.clone();
        let show_toast = show_toast.clone();
        move || {
            let todos = todos.clone();
            let current_list = current_list.clone();
            let authenticated = authenticated.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get("/api/todos").send().await {
                    Ok(resp) => {
                        if resp.status() == 401 {
                            authenticated.set(false);
                        } else if let Ok(data) = resp.json::<TodoLists>().await {
                            authenticated.set(true);
                            if !data.is_empty() {
                                // Keep current list if it still exists, otherwise set to the first list
                                if !data.contains_key(&*current_list) {
                                    if let Some(first_key) = data.keys().next() {
                                        current_list.set(first_key.clone());
                                    }
                                }
                            }
                            todos.set(Some(data));
                        } else {
                            show_toast("Failed to parse todo data".to_string(), ToastType::Error);
                        }
                    }
                    Err(_) => {
                        show_toast("Failed to fetch todos".to_string(), ToastType::Error);
                    }
                }
            });
        }
    };

    // Fetch initial setup configurations
    {
        let site_config = site_config.clone();
        let pin_required = pin_required.clone();
        let pin_digits = pin_digits.clone();
        let load_todos = load_todos.clone();
        let pin_refs = pin_refs.clone();
        use_effect_with((), move |_| {
            // Fetch Site config
            {
                let site_config = site_config.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(resp) = Request::get("/api/config").send().await {
                        if let Ok(config) = resp.json::<SiteConfig>().await {
                            site_config.set(Some(config));
                        }
                    }
                });
            }

            // Fetch PIN required details
            {
                let pin_required = pin_required.clone();
                let pin_digits = pin_digits.clone();
                let pin_refs = pin_refs.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(resp) = Request::get("/api/pin-required").send().await {
                        if let Ok(data) = resp.json::<PinRequiredResponse>().await {
                            let mut refs = Vec::new();
                            for _ in 0..data.length {
                                refs.push(NodeRef::default());
                            }
                            pin_refs.set(refs);
                            pin_digits.set(vec!["".to_string(); data.length]);
                            pin_required.set(Some(data));
                        }
                    }
                });
            }

            // Check if we are already authenticated by fetching todos
            load_todos();
        });
    }

    // Auto-focus first PIN input box when PIN dialog appears
    {
        let pin_required = pin_required.clone();
        let pin_refs = pin_refs.clone();
        use_effect_with(pin_required, move |_| {
            if let Some(first_ref) = pin_refs.first() {
                if let Some(input) = first_ref.cast::<HtmlInputElement>() {
                    let _ = input.focus();
                }
            }
        });
    }

    // Auto-focus inline edit input box
    {
        let editing_todo_id = editing_todo_id.clone();
        let edit_ref = edit_ref.clone();
        use_effect_with(editing_todo_id, move |_| {
            if let Some(input) = edit_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        });
    }

    // PIN Submit function
    let submit_pin = {
        let pin_digits = pin_digits.clone();
        let pin_error = pin_error.clone();
        let pin_required = pin_required.clone();
        let load_todos = load_todos.clone();
        let show_toast = show_toast.clone();
        move |pin: String| {
            let pin_digits = pin_digits.clone();
            let pin_error = pin_error.clone();
            let pin_required = pin_required.clone();
            let load_todos = load_todos.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::post("/api/verify-pin")
                    .json(&VerifyPinRequest { pin })
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<VerifyPinResponse>().await {
                            if data.valid {
                                pin_error.set(None);
                                load_todos();
                                show_toast("Authenticated successfully 🎉".to_string(), ToastType::Success);
                            } else {
                                pin_error.set(data.error.clone());
                                // Clear digits
                                if let Some(ref pr) = *pin_required {
                                    pin_digits.set(vec!["".to_string(); pr.length]);
                                }
                                // Reset status details if provided
                                if let Some(left) = data.attempts_left {
                                    let mut updated = (*pin_required).clone().unwrap();
                                    updated.attempts_left = left;
                                    if let Some(locked) = data.locked {
                                        updated.locked = locked;
                                    }
                                    if let Some(lockout) = data.lockout_minutes {
                                        updated.lockout_minutes = lockout;
                                    }
                                    pin_required.set(Some(updated));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        pin_error.set(Some("Connection failed".to_string()));
                    }
                }
            });
        }
    };

    // Handle PIN input box interaction
    let on_pin_input = {
        let pin_digits = pin_digits.clone();
        let pin_refs = pin_refs.clone();
        let submit_pin = submit_pin.clone();
        move |idx: usize, val: String| {
            let mut digits = (*pin_digits).clone();
            // Restrict to last character
            let clean_val = val.chars().last().map(|c| c.to_string()).unwrap_or_default();
            digits[idx] = clean_val;
            pin_digits.set(digits.clone());

            if !digits[idx].is_empty() && idx < pin_refs.len() - 1 {
                // Focus next
                if let Some(input) = pin_refs[idx + 1].cast::<HtmlInputElement>() {
                    let _ = input.focus();
                }
            }

            // Check if full PIN entered
            let full_pin = digits.join("");
            if full_pin.len() == pin_refs.len() {
                submit_pin(full_pin);
            }
        }
    };

    let on_pin_keydown = {
        let pin_digits = pin_digits.clone();
        let pin_refs = pin_refs.clone();
        move |idx: usize, e: KeyboardEvent| {
            if e.key() == "Backspace" {
                let mut digits = (*pin_digits).clone();
                
                if digits[idx].is_empty() && idx > 0 {
                    digits[idx - 1] = "".to_string();
                    pin_digits.set(digits);
                    
                    if let Some(input) = pin_refs[idx - 1].cast::<HtmlInputElement>() {
                        let _ = input.focus();
                    }
                } else {
                    digits[idx] = "".to_string();
                    pin_digits.set(digits);
                }
            }
        }
    };

    // Save Todos helper
    let save_list_todos = {
        let todos = todos.clone();
        let show_toast = show_toast.clone();
        move |updated_todos: TodoLists| {
            let todos = todos.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Request::post("/api/todos")
                    .json(&updated_todos)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            todos.set(Some(updated_todos));
                        } else {
                            show_toast("Failed to save changes".to_string(), ToastType::Error);
                        }
                    }
                    Err(_) => {
                        show_toast("Connection error, changes unsaved".to_string(), ToastType::Error);
                    }
                }
            });
        }
    };

    // Task Interactions
    let add_todo = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let form = e.target_dyn_into::<HtmlFormElement>().unwrap();
            let input_el = form.elements().get_with_name("todoInput").unwrap().dyn_into::<HtmlInputElement>().unwrap();
            let text = input_el.value().trim().to_string();
            
            if !text.is_empty() && todos.is_some() {
                let mut data = (*todos).clone().unwrap();
                let list = data.entry((*current_list).clone()).or_insert_with(Vec::new);
                
                // Generate a runtime unique ID using browser js_sys::Math API
                let unique_id = format!("{}-{}", js_sys::Date::now(), (js_sys::Math::random() * 1000000.0) as u32);
                list.push(TodoItem {
                    id: unique_id,
                    text,
                    completed: false,
                });
                
                save_list_todos(data);
                input_el.set_value("");
                show_toast("Task added".to_string(), ToastType::Success);
            }
        }
    };

    let toggle_todo = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |id: String| {
            if let Some(ref list_todos) = *todos {
                let mut data = list_todos.clone();
                if let Some(list) = data.get_mut(&*current_list) {
                    if let Some(item) = list.iter_mut().find(|t| t.id == id) {
                        item.completed = !item.completed;
                        let msg = if item.completed { "Task completed! 🎉" } else { "Task uncompleted" };
                        save_list_todos(data);
                        show_toast(msg.to_string(), ToastType::Success);
                    }
                }
            }
        }
    };

    let delete_todo = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |id: String, text: String| {
            let window = web_sys::window().unwrap();
            if window.confirm_with_message(&format!("Are you sure you want to delete \"{}\"?", text)).unwrap_or(false) {
                if let Some(ref list_todos) = *todos {
                    let mut data = list_todos.clone();
                    if let Some(list) = data.get_mut(&*current_list) {
                        list.retain(|t| t.id != id);
                        save_list_todos(data);
                        show_toast("Task deleted".to_string(), ToastType::Error);
                    }
                }
            }
        }
    };

    let clear_completed = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |_| {
            if let Some(ref list_todos) = *todos {
                if let Some(list) = list_todos.get(&*current_list) {
                    let completed_count = list.iter().filter(|t| t.completed).count();
                    if completed_count == 0 {
                        show_toast("No completed tasks to clear".to_string(), ToastType::Error);
                        return;
                    }
                    
                    let window = web_sys::window().unwrap();
                    let msg = format!("Are you sure you want to delete {} completed task{}?", completed_count, if completed_count == 1 { "" } else { "s" });
                    if window.confirm_with_message(&msg).unwrap_or(false) {
                        let mut data = list_todos.clone();
                        if let Some(l) = data.get_mut(&*current_list) {
                            l.retain(|t| !t.completed);
                            save_list_todos(data);
                            show_toast(format!("Cleared {} completed task{}", completed_count, if completed_count == 1 { "" } else { "s" }), ToastType::Success);
                        }
                    }
                }
            }
        }
    };

    // Inline edit triggers
    let start_edit = {
        let editing_todo_id = editing_todo_id.clone();
        let edit_input_value = edit_input_value.clone();
        move |id: String, val: String| {
            editing_todo_id.set(Some(id));
            edit_input_value.set(val);
        }
    };

    let save_edit = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let editing_todo_id = editing_todo_id.clone();
        let edit_input_value = edit_input_value.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move || {
            if let (Some(id), Some(ref list_todos)) = (&*editing_todo_id, &*todos) {
                let new_text = edit_input_value.trim().to_string();
                if !new_text.is_empty() {
                    let mut data = list_todos.clone();
                    if let Some(list) = data.get_mut(&*current_list) {
                        if let Some(item) = list.iter_mut().find(|t| t.id == *id) {
                            if item.text != new_text {
                                item.text = new_text;
                                save_list_todos(data);
                                show_toast("Task updated".to_string(), ToastType::Success);
                            }
                        }
                    }
                }
            }
            editing_todo_id.set(None);
        }
    };

    // List Selector Controls
    let switch_list = {
        let current_list = current_list.clone();
        let custom_select_open = custom_select_open.clone();
        move |list_name: String| {
            current_list.set(list_name);
            custom_select_open.set(false);
        }
    };

    let add_new_list = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |_| {
            if let Some(ref list_todos) = *todos {
                let mut data = list_todos.clone();
                let next_idx = data.len() + 1;
                let list_name = format!("List {}", next_idx);
                data.insert(list_name.clone(), Vec::new());
                current_list.set(list_name);
                save_list_todos(data);
                show_toast("New list added".to_string(), ToastType::Success);
            }
        }
    };

    let rename_list = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |_| {
            if let Some(ref list_todos) = *todos {
                let window = web_sys::window().unwrap();
                if let Ok(Some(new_name)) = window.prompt_with_message_and_default("Enter new list name:", &*current_list) {
                    let clean_name = new_name.trim().to_string();
                    if !clean_name.is_empty() && clean_name != *current_list && !list_todos.contains_key(&clean_name) {
                        let mut data = list_todos.clone();
                        if let Some(items) = data.remove(&*current_list) {
                            data.insert(clean_name.clone(), items);
                            current_list.set(clean_name);
                            save_list_todos(data);
                            show_toast("List renamed".to_string(), ToastType::Success);
                        }
                    }
                }
            }
        }
    };

    let delete_list = {
        let todos = todos.clone();
        let current_list = current_list.clone();
        let save_list_todos = save_list_todos.clone();
        let show_toast = show_toast.clone();
        move |list_name: String| {
            if list_name == "List 1" {
                show_toast("Cannot delete List 1".to_string(), ToastType::Error);
                return;
            }
            if let Some(ref list_todos) = *todos {
                if list_todos.len() <= 1 {
                    show_toast("Cannot delete the last list".to_string(), ToastType::Error);
                    return;
                }
                
                let window = web_sys::window().unwrap();
                let msg = format!("Are you sure you want to delete \"{}\" and all its tasks?", list_name);
                if window.confirm_with_message(&msg).unwrap_or(false) {
                    let mut data = list_todos.clone();
                    data.remove(&list_name);
                    
                    if *current_list == list_name {
                        if let Some(first_key) = data.keys().next() {
                            current_list.set(first_key.clone());
                        }
                    }
                    
                    save_list_todos(data);
                    show_toast("List deleted".to_string(), ToastType::Success);
                }
            }
        }
    };

    // Render list items dynamically (including drag and drop events)
    let render_todo_items = {
        let current_list = current_list.clone();
        let todos = todos.clone();
        let dragged_todo_id = dragged_todo_id.clone();
        let editing_todo_id = editing_todo_id.clone();
        let edit_input_value = edit_input_value.clone();
        let toggle_todo = toggle_todo.clone();
        let delete_todo = delete_todo.clone();
        let start_edit = start_edit.clone();
        let save_edit = save_edit.clone();
        let save_list_todos = save_list_todos.clone();
        let edit_ref = edit_ref.clone();

        move |is_completed: bool| {
            let current_list = current_list.clone();
            let todos = todos.clone();
            let dragged_todo_id = dragged_todo_id.clone();
            let editing_todo_id = editing_todo_id.clone();
            let edit_input_value = edit_input_value.clone();
            let toggle_todo = toggle_todo.clone();
            let delete_todo = delete_todo.clone();
            let start_edit = start_edit.clone();
            let save_edit = save_edit.clone();
            let save_list_todos = save_list_todos.clone();
            let edit_ref = edit_ref.clone();

            let items = if let Some(ref data) = *todos {
                data.get(&*current_list).cloned().unwrap_or_default()
            } else {
                Vec::new()
            };

            let filtered_items: Vec<TodoItem> = items
                .into_iter()
                .filter(|t| t.completed == is_completed)
                .collect();

            filtered_items.into_iter().map(move |item| {
                let id = item.id.clone();
                let text = item.text.clone();
                let completed = item.completed;

                // Drag and drop event handlers
                let ondragstart = {
                    let dragged_todo_id = dragged_todo_id.clone();
                    let id = id.clone();
                    Callback::from(move |_e: DragEvent| {
                        dragged_todo_id.set(Some(id.clone()));
                    })
                };

                let ondragend = {
                    let dragged_todo_id = dragged_todo_id.clone();
                    Callback::from(move |_e: DragEvent| {
                        dragged_todo_id.set(None);
                    })
                };

                let ondragover = Callback::from(|e: DragEvent| {
                    e.prevent_default(); // Necessary to allow dropping
                });

                let ondrop = {
                    let current_list = current_list.clone();
                    let todos = todos.clone();
                    let dragged_todo_id = dragged_todo_id.clone();
                    let target_id = id.clone();
                    let save_list_todos = save_list_todos.clone();
                    Callback::from(move |e: DragEvent| {
                        e.prevent_default();
                        if let (Some(ref drag_id), Some(ref list_todos)) = (&*dragged_todo_id, &*todos) {
                            if *drag_id != target_id {
                                let mut data = list_todos.clone();
                                if let Some(list) = data.get_mut(&*current_list) {
                                    let drag_idx = list.iter().position(|t| t.id == *drag_id);
                                    let target_idx = list.iter().position(|t| t.id == target_id);
                                    if let (Some(di), Some(ti)) = (drag_idx, target_idx) {
                                        let item = list.remove(di);
                                        list.insert(ti, item);
                                        save_list_todos(data);
                                    }
                                }
                            }
                        }
                    })
                };

                let toggle = {
                    let toggle_todo = toggle_todo.clone();
                    let id = id.clone();
                    move |_| toggle_todo(id.clone())
                };

                let delete = {
                    let delete_todo = delete_todo.clone();
                    let id = id.clone();
                    let text = text.clone();
                    move |_| delete_todo(id.clone(), text.clone())
                };

                let start_editing = {
                    let start_edit = start_edit.clone();
                    let id = id.clone();
                    let text = text.clone();
                    move |_| start_edit(id.clone(), text.clone())
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
                            if Some(id.clone()) == *editing_todo_id {
                                let onblur = {
                                    let save_edit = save_edit.clone();
                                    move |_| save_edit()
                                };
                                let oninput = {
                                    let edit_input_value = edit_input_value.clone();
                                    move |e: InputEvent| {
                                        let el = e.target_dyn_into::<HtmlInputElement>().unwrap();
                                        edit_input_value.set(el.value());
                                    }
                                };
                                let onkeydown = {
                                    let save_edit = save_edit.clone();
                                    let editing_todo_id = editing_todo_id.clone();
                                    move |e: KeyboardEvent| {
                                        if e.key() == "Enter" {
                                            save_edit();
                                        } else if e.key() == "Escape" {
                                            editing_todo_id.set(None);
                                        }
                                    }
                                };
                                html! {
                                    <input 
                                        ref={edit_ref.clone()}
                                        type="text" 
                                        class="edit-input" 
                                        value={(*edit_input_value).clone()}
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
            }).collect::<Html>()
        }
    };

    // Dynamic URL Parser helper for task texts
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

    // Authenticated View
    let render_app_view = {
        let site_config = site_config.clone();
        let current_list = current_list.clone();
        let todos = todos.clone();
        let custom_select_open = custom_select_open.clone();
        let add_todo = add_todo.clone();
        let toggle_theme = toggle_theme.clone();
        let add_new_list = add_new_list.clone();
        let rename_list = rename_list.clone();
        let delete_list = delete_list.clone();
        let switch_list = switch_list.clone();
        let clear_completed = clear_completed.clone();
        let render_todo_items = render_todo_items.clone();
        let theme = theme.clone();

        move || {
            let config = site_config.as_ref().cloned().unwrap_or(SiteConfig {
                site_title: "RustDo".to_string(),
                single_list: false,
            });

            let lists_keys = if let Some(ref data) = *todos {
                let mut keys: Vec<String> = data.keys().cloned().collect();
                keys.sort_by(|a, b| {
                    if a == "List 1" { return std::cmp::Ordering::Less; }
                    if b == "List 1" { return std::cmp::Ordering::Greater; }
                    a.cmp(b)
                });
                keys
            } else {
                vec!["List 1".to_string()]
            };

            let current_list_todos = todos.as_ref()
                .and_then(|d| d.get(&*current_list))
                .cloned()
                .unwrap_or_default();
            
            let active_count = current_list_todos.iter().filter(|t| !t.completed).count();
            let completed_count = current_list_todos.iter().filter(|t| t.completed).count();

            html! {
                <div class="app">
                    <header>
                        <h1 id="header-title">{ &config.site_title }</h1>
                        
                        if !config.single_list {
                            <div class="list-controls" id="listControls">
                                <div class="list-selector-container">
                                    <select 
                                        id="listSelector" 
                                        aria-label="Select todo list"
                                        value={(*current_list).clone()}
                                        onchange={
                                            let switch_list = switch_list.clone();
                                            move |e: Event| {
                                                let el = e.target_dyn_into::<HtmlSelectElement>().unwrap();
                                                switch_list(el.value());
                                            }
                                        }
                                    >
                                        {
                                            lists_keys.iter().map(|list_id| html! {
                                                <option value={list_id.clone()} selected={list_id == &*current_list}>{list_id}</option>
                                            }).collect::<Html>()
                                        }
                                    </select>
                                    
                                    <div 
                                        class="custom-select" 
                                        style={if *custom_select_open { "display: block;" } else { "display: none;" }}
                                    >
                                        {
                                            lists_keys.iter().map(|list_id| {
                                                let switch = switch_list.clone();
                                                let del = delete_list.clone();
                                                let id = list_id.clone();
                                                let is_list_1 = list_id == "List 1";
                                                
                                                html! {
                                                    <div 
                                                        class={format!("list-item {}", if is_list_1 { "list-1" } else { "" })}
                                                        onclick={move |_| switch(id.clone())}
                                                    >
                                                        <span>{list_id}</span>
                                                        if !is_list_1 {
                                                            <button 
                                                                type="button" 
                                                                class="delete-btn" 
                                                                aria-label={format!("Delete {}", list_id)}
                                                                onclick={
                                                                    let id = list_id.clone();
                                                                    move |ev: MouseEvent| {
                                                                        ev.stop_propagation();
                                                                        del(id.clone());
                                                                    }
                                                                }
                                                            >
                                                                <svg viewBox="0 0 24 24">
                                                                    <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                                                </svg>
                                                            </button>
                                                        }
                                                    </div>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </div>
                                </div>
                                <div class="list-buttons">
                                    <button 
                                        type="button" 
                                        id="renameList" 
                                        class="icon-btn" 
                                        aria-label="Rename current list"
                                        onclick={rename_list}
                                    >
                                        <svg viewBox="0 0 24 24" width="14" height="14">
                                            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                                            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                                        </svg>
                                    </button>
                                    <button 
                                        type="button" 
                                        id="addList" 
                                        class="icon-btn" 
                                        aria-label="Add new list"
                                        onclick={add_new_list}
                                    >
                                        <svg viewBox="0 0 24 24" width="14" height="14">
                                            <line x1="12" y1="5" x2="12" y2="19"></line>
                                            <line x1="5" y1="12" x2="19" y2="12"></line>
                                        </svg>
                                    </button>
                                </div>
                                <button 
                                    type="button" 
                                    style="margin-left: auto; font-size: 0.8rem; padding: 0.25rem 0.5rem;"
                                    onclick={
                                        let custom_select_open = custom_select_open.clone();
                                        move |_| custom_select_open.set(!*custom_select_open)
                                    }
                                >
                                    { if *custom_select_open { "Hide Lists" } else { "Manage Lists" } }
                                </button>
                            </div>
                        }

                        <button id="themeToggle" aria-label="Toggle theme" onclick={toggle_theme}>
                            if *theme == "dark" {
                                <svg class="sun" viewBox="0 0 24 24" style="display: block;">
                                    <circle cx="12" cy="12" r="5"></circle>
                                    <line x1="12" y1="1" x2="12" y2="3"></line>
                                    <line x1="12" y1="21" x2="12" y2="23"></line>
                                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                                    <line x1="1" y1="12" x2="3" y2="12"></line>
                                    <line x1="21" y1="12" x2="23" y2="12"></line>
                                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                                </svg>
                            } else {
                                <svg class="moon" viewBox="0 0 24 24" style="display: block;">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            }
                        </button>
                    </header>
                    
                    <main>
                        <form id="todoForm" class="todo-form" onsubmit={add_todo}>
                            <input 
                                type="text" 
                                name="todoInput"
                                id="todoInput" 
                                placeholder="What needs to be done?"
                                required=true
                            />
                            <div class="button-group">
                                <button type="submit">{"Add"}</button>
                                <button type="button" id="clearCompleted" class="clear-btn" aria-label="Delete completed tasks" onclick={clear_completed}>
                                    <svg viewBox="0 0 24 24" width="16" height="16">
                                        <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                    </svg>
                                </button>
                            </div>
                        </form>

                        <ul id="todoList" class="todo-list">
                            <div class="active-todos">
                                { render_todo_items(false) }
                            </div>
                            
                            if active_count > 0 && completed_count > 0 {
                                <li class="todo-divider">{"Completed"}</li>
                            }
                            
                            { render_todo_items(true) }
                        </ul>
                    </main>
                </div>
            }
        }
    };

    // Unauthenticated PIN/Login View
    let render_login_view = {
        let pin_digits = pin_digits.clone();
        let pin_error = pin_error.clone();
        let pin_required = pin_required.clone();
        let on_pin_input = on_pin_input.clone();
        let on_pin_keydown = on_pin_keydown.clone();
        let toggle_theme = toggle_theme.clone();
        let theme = theme.clone();
        let pin_refs = pin_refs.clone();

        move || {
            let pr = pin_required.as_ref().cloned().unwrap_or(PinRequiredResponse {
                required: true,
                length: 4,
                locked: false,
                attempts_left: 5,
                lockout_minutes: 0,
            });

            html! {
                <div class="app login-page">
                    <div class="login-container">
                        <h1>{"RustDo"}</h1>
                        <div class="login-box">
                            <h2>{"Enter PIN"}</h2>
                            <p id="pin-description" class="pin-description">
                                {
                                    if pr.locked {
                                        format!("Locked out. Try again in {} minutes.", pr.lockout_minutes)
                                    } else {
                                        "Please enter your PIN to access your todos.".to_string()
                                    }
                                }
                            </p>
                            <form id="loginForm" onsubmit={Callback::from(|e: SubmitEvent| e.prevent_default())}>
                                <div class="pin-input-container">
                                    {
                                        (0..pr.length).map(|i| {
                                            let oninput = {
                                                let on_pin_input = on_pin_input.clone();
                                                move |e: InputEvent| {
                                                    let el = e.target_dyn_into::<HtmlInputElement>().unwrap();
                                                    on_pin_input(i, el.value());
                                                }
                                            };
                                            let onkeydown = {
                                                let on_pin_keydown = on_pin_keydown.clone();
                                                move |e: KeyboardEvent| {
                                                    on_pin_keydown(i, e);
                                                }
                                            };
                                            
                                            let current_digit_val = pin_digits.get(i).cloned().unwrap_or_default();
                                            let has_val = !current_digit_val.is_empty();
                                            let node_ref = pin_refs.get(i).cloned().unwrap_or_default();
                                            
                                            html! {
                                                <input 
                                                    ref={node_ref}
                                                    type="password" 
                                                    maxlength="1" 
                                                    pattern="[0-9]" 
                                                    inputmode="numeric" 
                                                    class={format!("pin-input {}", if has_val { "has-value" } else { "" })}
                                                    aria-label={format!("PIN digit {}", i + 1)}
                                                    value={current_digit_val}
                                                    disabled={pr.locked}
                                                    {oninput}
                                                    {onkeydown}
                                                />
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                                <div class="pin-status">
                                    if pr.locked {
                                        <p id="lockoutNotice" class="lockout-notice" style="display: block;">
                                            { format!("Too many attempts. Locked out for {} minutes.", pr.lockout_minutes) }
                                        </p>
                                    } else {
                                        if pr.attempts_left < 5 {
                                            <p id="attemptsRemaining" class="attempts-remaining" style="display: block;">
                                                { format!("{} attempt{} remaining", pr.attempts_left, if pr.attempts_left == 1 { "" } else { "s" }) }
                                            </p>
                                        }
                                    }
                                    
                                    if let Some(ref err) = *pin_error {
                                        <p id="pinError" class="pin-error" style="display: block;">{ err }</p>
                                    }
                                </div>
                            </form>
                        </div>
                        <button id="themeToggle" aria-label="Toggle theme" onclick={toggle_theme}>
                            if *theme == "dark" {
                                <svg class="sun" viewBox="0 0 24 24" style="display: block;">
                                    <circle cx="12" cy="12" r="5"></circle>
                                    <line x1="12" y1="1" x2="12" y2="3"></line>
                                    <line x1="12" y1="21" x2="12" y2="23"></line>
                                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                                    <line x1="1" y1="12" x2="3" y2="12"></line>
                                    <line x1="21" y1="12" x2="23" y2="12"></line>
                                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                                </svg>
                            } else {
                                <svg class="moon" viewBox="0 0 24 24" style="display: block;">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            }
                        </button>
                    </div>
                </div>
            }
        }
    };

    // Render Layout
    html! {
        <>
            if *authenticated || pin_required.as_ref().map(|pr| !pr.required).unwrap_or(true) {
                { render_app_view() }
            } else {
                { render_login_view() }
            }
            
            // Toasts container
            <div id="toast-container" class="toast-container">
                {
                    toasts.iter().map(|toast| html! {
                        <div key={toast.id} class={format!("toast show {}", match toast.toast_type {
                            ToastType::Success => "success",
                            ToastType::Error => "error"
                        })}>
                            { &toast.message }
                        </div>
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
