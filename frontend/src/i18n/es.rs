use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "Ingrese el PIN".to_string(),
        TransKey::LockedOut => "Bloqueado".to_string(),
        TransKey::PinDescription => {
            "Por favor ingrese su PIN para acceder a sus tareas.".to_string()
        }
        TransKey::AttemptsRemaining(n) => format!(
            "{} intento{} restante{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
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
        TransKey::ConfirmDeleteTask(t) => {
            format!("¿Estás seguro de que quieres eliminar \"{}\"?", t)
        }
        TransKey::ConfirmDeleteCompleted(n) => format!(
            "¿Eliminar {} tarea{} completada{}?",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
        TransKey::ConfirmDeleteList(l) => format!("¿Eliminar \"{}\" y todas sus tareas?", l),
        TransKey::PromptRenameList => "Ingrese el nuevo nombre de la lista:".to_string(),
        TransKey::ThemeToggle => "Cambiar tema".to_string(),
        TransKey::ClearedCompleted(n) => format!(
            "Se limpiaron {} tarea{} completada{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
    }
}
