use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "Inserir PIN".to_string(),
        TransKey::LockedOut => "Bloqueado".to_string(),
        TransKey::PinDescription => {
            "Por favor, insira o seu PIN para aceder às suas tarefas.".to_string()
        }
        TransKey::AttemptsRemaining(n) => format!(
            "{} tentativa{} restante{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
        TransKey::LockoutNotice(m) => {
            format!("Demasiadas tentativas. Bloqueado por {} minutos.", m)
        }
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "O que precisa de ser feito?".to_string(),
        TransKey::Add => "Adicionar".to_string(),
        TransKey::DeleteCompleted => "Eliminar tarefas concluídas".to_string(),
        TransKey::CompletedHeader => "Concluído".to_string(),
        TransKey::NoCompletedTasks => "Nenhuma tarefa concluída para limpar".to_string(),
        TransKey::TaskAdded => "Tarefa adicionada".to_string(),
        TransKey::TaskCompleted => "Tarefa concluída! 🎉".to_string(),
        TransKey::TaskUncompleted => "Tarefa não concluída".to_string(),
        TransKey::TaskDeleted => "Tarefa eliminada".to_string(),
        TransKey::TaskUpdated => "Tarefa atualizada".to_string(),
        TransKey::NewListAdded => "Nova lista adicionada".to_string(),
        TransKey::ListRenamed => "Lista renomeada".to_string(),
        TransKey::ListDeleted => "Lista eliminada".to_string(),
        TransKey::RenameCurrentList => "Renombrar lista atual".to_string(),
        TransKey::AddNewList => "Adicionar nova lista".to_string(),
        TransKey::HideLists => "Ocultar listas".to_string(),
        TransKey::ManageLists => "Gerir listas".to_string(),
        TransKey::ConfirmDeleteTask(t) => {
            format!("Tem a certeza de que deseja eliminar \"{}\"?", t)
        }
        TransKey::ConfirmDeleteCompleted(n) => format!(
            "Eliminar {} tarefa{} concluída{}?",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
        TransKey::ConfirmDeleteList(l) => format!("Eliminar \"{}\" e todas as suas tarefas?", l),
        TransKey::PromptRenameList => "Introduza o novo nome da lista:".to_string(),
        TransKey::ThemeToggle => "Alternar tema".to_string(),
        TransKey::ClearedCompleted(n) => format!(
            "{} tarefa{} concluída{} eliminada{}",
            n,
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" },
            if n == 1 { "" } else { "s" }
        ),
    }
}
