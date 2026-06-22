use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "Saisir le code PIN".to_string(),
        TransKey::LockedOut => "Verrouillé".to_string(),
        TransKey::PinDescription => {
            "Veuillez saisir votre code PIN pour accéder à vos tâches.".to_string()
        }
        TransKey::AttemptsRemaining(n) => format!(
            "{} tentative{} restante{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
        TransKey::LockoutNotice(m) => {
            format!("Trop de tentatives. Verrouillé pendant {} minutes.", m)
        }
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "Que faut-il faire ?".to_string(),
        TransKey::Add => "Ajouter".to_string(),
        TransKey::DeleteCompleted => "Supprimer les tâches terminées".to_string(),
        TransKey::CompletedHeader => "Terminé".to_string(),
        TransKey::NoCompletedTasks => "Aucune tâche terminée à effacer".to_string(),
        TransKey::TaskAdded => "Tâche ajoutée".to_string(),
        TransKey::TaskCompleted => "Tâche terminée ! 🎉".to_string(),
        TransKey::TaskUncompleted => "Tâche non terminée".to_string(),
        TransKey::TaskDeleted => "Tâche supprimée".to_string(),
        TransKey::TaskUpdated => "Tâche mise à jour".to_string(),
        TransKey::NewListAdded => "Nouvelle liste ajoutée".to_string(),
        TransKey::ListRenamed => "Liste renommée".to_string(),
        TransKey::ListDeleted => "Liste supprimée".to_string(),
        TransKey::RenameCurrentList => "Renommer la liste actuelle".to_string(),
        TransKey::AddNewList => "Ajouter une nouvelle liste".to_string(),
        TransKey::HideLists => "Masquer les listes".to_string(),
        TransKey::ManageLists => "Gérer les listes".to_string(),
        TransKey::ConfirmDeleteTask(t) => format!("Êtes-vous sûr de vouloir supprimer « {} » ?", t),
        TransKey::ConfirmDeleteCompleted(n) => format!(
            "Supprimer {} tâche{} terminée{} ?",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
        TransKey::ConfirmDeleteList(l) => format!("Supprimer « {} » et toutes ses tâches ?", l),
        TransKey::PromptRenameList => "Saisir le nouveau nom de la liste :".to_string(),
        TransKey::ThemeToggle => "Changer de thème".to_string(),
        TransKey::ClearedCompleted(n) => format!(
            "{} tâche{} terminée{} effacée{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
    }
}
