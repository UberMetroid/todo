use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    En,
    Es,
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "es" => Self::Es,
            _ => Self::En,
        }
    }
    
    pub fn to_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }
}

#[allow(dead_code)]
pub enum TransKey {
    EnterPin,
    LockedOut,
    PinDescription,
    AttemptsRemaining(usize),
    LockoutNotice(usize),
    PinInputPlaceholder(usize),
    WhatNeedsBeDone,
    Add,
    DeleteCompleted,
    CompletedHeader,
    NoCompletedTasks,
    TaskAdded,
    TaskCompleted,
    TaskUncompleted,
    TaskDeleted,
    TaskUpdated,
    NewListAdded,
    ListRenamed,
    ListDeleted,
    RenameCurrentList,
    AddNewList,
    HideLists,
    ManageLists,
    ConfirmDeleteTask(String),
    ConfirmDeleteCompleted(usize),
    ConfirmDeleteList(String),
    PromptRenameList,
    ThemeToggle,
    ClearedCompleted(usize),
}

pub fn translate(locale: Locale, key: TransKey) -> String {
    match locale {
        Locale::En => match key {
            TransKey::EnterPin => "Enter PIN".to_string(),
            TransKey::LockedOut => "Locked Out".to_string(),
            TransKey::PinDescription => "Please enter your PIN to access your todos.".to_string(),
            TransKey::AttemptsRemaining(n) => format!("{} attempt{} remaining", n, if n == 1 { "" } else { "s" }),
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
            TransKey::ConfirmDeleteCompleted(n) => format!("Delete {} completed task{}?", n, if n == 1 { "" } else { "s" }),
            TransKey::ConfirmDeleteList(l) => format!("Delete \"{}\" and all its tasks?", l),
            TransKey::PromptRenameList => "Enter new list name:".to_string(),
            TransKey::ThemeToggle => "Toggle theme".to_string(),
            TransKey::ClearedCompleted(n) => format!("Cleared {} completed task{}", n, if n == 1 { "" } else { "s" }),
        },
        Locale::Es => match key {
            TransKey::EnterPin => "Ingrese el PIN".to_string(),
            TransKey::LockedOut => "Bloqueado".to_string(),
            TransKey::PinDescription => "Por favor ingrese su PIN para acceder a sus tareas.".to_string(),
            TransKey::AttemptsRemaining(n) => format!("{} intento{} restante{}", n, if n == 1 { "" } else { "s" }, if n == 1 { "" } else { "s" }),
            TransKey::LockoutNotice(m) => format!("Demasiados intentos. Bloqueado por {} minutos.", m),
            TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
            TransKey::WhatNeedsBeDone => "¿Qué hay que hacer?".to_string(),
            TransKey::Add => "Añadir".to_string(),
            TransKey::DeleteCompleted => "Eliminar tareas completadas".to_string(),
            TransKey::CompletedHeader => "Completado".to_string(),
            TransKey::NoCompletedTasks => "No hay tareas completadas para limpiar".to_string(),
            TransKey::TaskAdded => "Tarea añadida".to_string(),
            TransKey::TaskCompleted => "¡Tarea completada! 🎉".to_string(),
            TransKey::TaskUncompleted => "Tarea desmarcada".to_string(),
            TransKey::TaskDeleted => "Tarea eliminada".to_string(),
            TransKey::TaskUpdated => "Tarea actualizada".to_string(),
            TransKey::NewListAdded => "Nueva lista añadida".to_string(),
            TransKey::ListRenamed => "Lista renombrada".to_string(),
            TransKey::ListDeleted => "Lista eliminada".to_string(),
            TransKey::RenameCurrentList => "Renombrar lista actual".to_string(),
            TransKey::AddNewList => "Añadir nueva lista".to_string(),
            TransKey::HideLists => "Ocultar listas".to_string(),
            TransKey::ManageLists => "Gestionar listas".to_string(),
            TransKey::ConfirmDeleteTask(t) => format!("¿Estás seguro de que quieres eliminar \"{}\"?", t),
            TransKey::ConfirmDeleteCompleted(n) => format!("¿Eliminar {} tarea{} completada{}?", n, if n == 1 { "" } else { "s" }, if n == 1 { "" } else { "s" }),
            TransKey::ConfirmDeleteList(l) => format!("¿Eliminar \"{}\" y todas sus tareas?", l),
            TransKey::PromptRenameList => "Ingrese el nuevo nombre de la lista:".to_string(),
            TransKey::ThemeToggle => "Cambiar tema".to_string(),
            TransKey::ClearedCompleted(n) => format!("Se limpiaron {} tarea{} completada{}", n, if n == 1 { "" } else { "s" }, if n == 1 { "" } else { "s" }),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Translator {
    pub locale: Locale,
}

impl Translator {
    pub fn t(&self, key: TransKey) -> String {
        translate(self.locale, key)
    }
}

pub type I18nContext = UseStateHandle<Locale>;

#[hook]
pub fn use_i18n() -> (Locale, Callback<Locale>, Translator) {
    let locale_handle = use_context::<I18nContext>().expect("I18nContext not found");
    let locale = *locale_handle;
    let set_locale = Callback::from(move |l: Locale| locale_handle.set(l));
    let t = Translator { locale };
    (locale, set_locale, t)
}
