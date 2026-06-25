// Adam Yew Frontend Entrypoint
//
// This is the main entry point for the Adam web application.
// It initializes and renders the main App component using Yew.
//
// Copyright (c) 2026 Adam Authors. All rights reserved.

mod api;
mod app;
mod header;
mod i18n;
mod login;
mod storage;
mod theme;
mod todo_form;
mod todo_item;
mod todo_items_list;
mod todo_list;
mod todo_list_handlers;
mod types;

fn main() {
    // Initialize logging or other runtime settings if needed in the future
    yew::Renderer::<app::App>::new().render();
}
