use yew::prelude::*;
use web_sys::{HtmlSelectElement, MouseEvent};

#[derive(Properties, PartialEq)]
pub struct ListSelectorProps {
    pub current_list: String,
    pub lists_keys: Vec<String>,
    pub custom_select_open: UseStateHandle<bool>,
    pub on_list_change: Callback<String>,
    pub on_delete_list: Callback<String>,
    pub on_rename_list: Callback<()>,
    pub on_add_list: Callback<()>,
}

// Renders list selectors (native select dropdown, styled custom dropdown container, delete buttons, and edit actions)
#[function_component(ListSelector)]
pub fn list_selector(props: &ListSelectorProps) -> Html {
    let current_list = &props.current_list;
    let custom_select_open = props.custom_select_open.clone();
    let is_open = *custom_select_open;
    let lists_keys = &props.lists_keys;

    let on_list_change = props.on_list_change.clone();
    let on_delete_list = props.on_delete_list.clone();
    let on_rename_list = props.on_rename_list.clone();
    let on_add_list = props.on_add_list.clone();

    html! {
        <div class="list-controls" id="listControls">
            <div class="list-selector-container">
                <select 
                    id="listSelector" 
                    aria-label="Select todo list"
                    value={current_list.clone()}
                    onchange={
                        let on_change = on_list_change.clone();
                        move |e: Event| {
                            let el = e.target_dyn_into::<HtmlSelectElement>().unwrap();
                            on_change.emit(el.value());
                        }
                    }
                >
                    {
                        lists_keys.iter().map(|list_id| html! {
                            <option value={list_id.clone()} selected={list_id == current_list}>{list_id}</option>
                        }).collect::<Html>()
                    }
                </select>
                
                <div class="custom-select" style={if is_open { "display: block;" } else { "display: none;" }}>
                    {
                        lists_keys.iter().map(|list_id| {
                            let switch = on_list_change.clone();
                            let del = on_delete_list.clone();
                            let custom_select_open = custom_select_open.clone();
                            let id = list_id.clone();
                            let is_list_1 = list_id == "List 1";
                            html! {
                                <div class={format!("list-item {}", if is_list_1 { "list-1" } else { "" })} onclick={move |_| { switch.emit(id.clone()); custom_select_open.set(false); } }>
                                    <span>{list_id}</span>
                                    if !is_list_1 {
                                        <button type="button" class="delete-btn" aria-label={format!("Delete {}", list_id)} onclick={ let id = list_id.clone(); move |ev: MouseEvent| { ev.stop_propagation(); del.emit(id.clone()); } }>
                                            <svg viewBox="0 0 24 24"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                        </button>
                                    }
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>
            
            <div class="list-buttons">
                <button type="button" id="renameList" class="icon-btn" aria-label="Rename current list" onclick={move |_| on_rename_list.emit(())}><svg viewBox="0 0 24 24" width="14" height="14"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg></button>
                <button type="button" id="addList" class="icon-btn" aria-label="Add new list" onclick={move |_| on_add_list.emit(())}><svg viewBox="0 0 24 24" width="14" height="14"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg></button>
            </div>
            
            <button type="button" style="margin-left: auto; font-size: 0.8rem; padding: 0.25rem 0.5rem;" onclick={ let custom_select_open = custom_select_open.clone(); move |_| custom_select_open.set(!*custom_select_open) }>
                { if is_open { "Hide Lists" } else { "Manage Lists" } }
            </button>
        </div>
    }
}
