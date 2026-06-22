use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "PIN eingeben".to_string(),
        TransKey::LockedOut => "Gesperrt".to_string(),
        TransKey::PinDescription => {
            "Bitte geben Sie Ihre PIN ein, um auf Ihre Aufgaben zuzugreifen.".to_string()
        }
        TransKey::AttemptsRemaining(n) => {
            format!("{} Versuch{} verbleibend", n, if n == 1 { "" } else { "e" })
        }
        TransKey::LockoutNotice(m) => format!("Zu viele Versuche. Für {} Minuten gesperrt.", m),
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "Was muss getan werden?".to_string(),
        TransKey::Add => "Hinzufügen".to_string(),
        TransKey::DeleteCompleted => "Erledigte Aufgaben löschen".to_string(),
        TransKey::CompletedHeader => "Erledigt".to_string(),
        TransKey::NoCompletedTasks => "Keine erledigten Aufgaben zum Löschen vorhanden".to_string(),
        TransKey::TaskAdded => "Aufgabe hinzugefügt".to_string(),
        TransKey::TaskCompleted => "Aufgabe erledigt! 🎉".to_string(),
        TransKey::TaskUncompleted => "Aufgabe als unerledigt markiert".to_string(),
        TransKey::TaskDeleted => "Aufgabe gelöscht".to_string(),
        TransKey::TaskUpdated => "Aufgabe aktualisiert".to_string(),
        TransKey::NewListAdded => "Neue Liste hinzugefügt".to_string(),
        TransKey::ListRenamed => "Liste umbenannt".to_string(),
        TransKey::ListDeleted => "Liste gelöscht".to_string(),
        TransKey::RenameCurrentList => "Aktuelle Liste umbenennen".to_string(),
        TransKey::AddNewList => "Neue Liste hinzufügen".to_string(),
        TransKey::HideLists => "Listen ausblenden".to_string(),
        TransKey::ManageLists => "Listen verwalten".to_string(),
        TransKey::ConfirmDeleteTask(t) => {
            format!("Sind Sie sicher, dass Sie \"{}\" löschen möchten?", t)
        }
        TransKey::ConfirmDeleteCompleted(n) => format!(
            "{} erledigte Aufgabe{} löschen?",
            n,
            if n == 1 { "" } else { "n" }
        ),
        TransKey::ConfirmDeleteList(l) => format!(
            "Möchten Sie \"{}\" und alle darin enthaltenen Aufgaben löschen?",
            l
        ),
        TransKey::PromptRenameList => "Geben Sie einen neuen Listennamen ein:".to_string(),
        TransKey::ThemeToggle => "Design umschalten".to_string(),
        TransKey::ClearedCompleted(n) => format!(
            "{} erledigte Aufgabe{} gelöscht",
            n,
            if n == 1 { "" } else { "n" }
        ),
    }
}
