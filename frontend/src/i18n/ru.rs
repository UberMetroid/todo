use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "Введите PIN".to_string(),
        TransKey::LockedOut => "Заблокировано".to_string(),
        TransKey::PinDescription => {
            "Пожалуйста, введите свой PIN-код для доступа к задачам.".to_string()
        }
        TransKey::AttemptsRemaining(n) => format!("Осталось попыток: {}", n),
        TransKey::LockoutNotice(m) => {
            format!("Слишком много попыток. Доступ заблокирован на {} мин.", m)
        }
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "Что нужно сделать?".to_string(),
        TransKey::Add => "Добавить".to_string(),
        TransKey::DeleteCompleted => "Удалить завершенные задачи".to_string(),
        TransKey::CompletedHeader => "Завершено".to_string(),
        TransKey::NoCompletedTasks => "Нет завершенных задач для удаления".to_string(),
        TransKey::TaskAdded => "Задача добавлена".to_string(),
        TransKey::TaskCompleted => "Задача выполнена! 🎉".to_string(),
        TransKey::TaskUncompleted => "Выполнение задачи отменено".to_string(),
        TransKey::TaskDeleted => "Задача удалена".to_string(),
        TransKey::TaskUpdated => "Задача обновлена".to_string(),
        TransKey::NewListAdded => "Добавлен новый список".to_string(),
        TransKey::ListRenamed => "Список переименован".to_string(),
        TransKey::ListDeleted => "Список удален".to_string(),
        TransKey::RenameCurrentList => "Переименовать текущий список".to_string(),
        TransKey::AddNewList => "Добавить новый список".to_string(),
        TransKey::HideLists => "Скрыть списки".to_string(),
        TransKey::ManageLists => "Управление списками".to_string(),
        TransKey::ConfirmDeleteTask(t) => format!("Вы уверены, что хотите удалить задачу «{}»?", t),
        TransKey::ConfirmDeleteCompleted(n) => format!("Удалить завершенные задачи ({})?", n),
        TransKey::ConfirmDeleteList(l) => format!("Удалить список «{}» со всеми его задачами?", l),
        TransKey::PromptRenameList => "Введите новое название списка:".to_string(),
        TransKey::ThemeToggle => "Переключить тему".to_string(),
        TransKey::ClearedCompleted(n) => format!("Удалено завершенных задач: {}", n),
    }
}
