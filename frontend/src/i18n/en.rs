use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "Enter PIN".to_string(),
        TransKey::LockedOut => "Locked Out".to_string(),
        TransKey::PinDescription => "Please enter your PIN to access your todos.".to_string(),
        TransKey::AttemptsRemaining(n) => {
            format!("{} attempt{} remaining", n, if n == 1 { "" } else { "s" })
        }
        TransKey::LockoutNotice(m) => format!("Too many attempts. Locked out for {} minutes.", m),
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "What needs to be done?".to_string(),
        TransKey::Add => "Add".to_string(),
        TransKey::DeleteCompleted => "Delete completed tasks".to_string(),
        TransKey::CompletedHeader => "Completed".to_string(),
        TransKey::NoCompletedTasks => "No completed tasks to clear".to_string(),
        TransKey::TaskAdded => "Task added".to_string(),
        TransKey::TaskCompleted => "Task completed! 🎉".to_string(),
        TransKey::TaskUncompleted => "Task uncompleted".to_string(),
        TransKey::TaskDeleted => "Task deleted".to_string(),
        TransKey::TaskUpdated => "Task updated".to_string(),
        TransKey::NewListAdded => "New list added".to_string(),
        TransKey::ListRenamed => "List renamed".to_string(),
        TransKey::ListDeleted => "List deleted".to_string(),
        TransKey::RenameCurrentList => "Rename current list".to_string(),
        TransKey::AddNewList => "Add new list".to_string(),
        TransKey::HideLists => "Hide Lists".to_string(),
        TransKey::ManageLists => "Manage Lists".to_string(),
        TransKey::ConfirmDeleteTask(t) => format!("Are you sure you want to delete \"{}\"?", t),
        TransKey::ConfirmDeleteCompleted(n) => format!(
            "Delete {} completed task{}?",
            n,
            if n == 1 { "" } else { "s" }
        ),
        TransKey::ConfirmDeleteList(l) => format!("Delete \"{}\" and all its tasks?", l),
        TransKey::PromptRenameList => "Enter new list name:".to_string(),
        TransKey::ThemeToggle => "Toggle theme".to_string(),
        TransKey::ClearedCompleted(n) => format!(
            "Cleared {} completed task{}",
            n,
            if n == 1 { "" } else { "s" }
        ),
    }
}
