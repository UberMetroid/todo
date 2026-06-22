use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    match key {
        TransKey::EnterPin => "PINを入力".to_string(),
        TransKey::LockedOut => "ロックアウト中".to_string(),
        TransKey::PinDescription => "タスクを表示するにはPINを入力してください。".to_string(),
        TransKey::AttemptsRemaining(n) => format!("残り試行回数: {}回", n),
        TransKey::LockoutNotice(m) => {
            format!("試行回数が上限を超えました。{}分間ロックされます。", m)
        }
        TransKey::PinInputPlaceholder(len) => "• ".repeat(len).trim().to_string(),
        TransKey::WhatNeedsBeDone => "何か用件はありますか？".to_string(),
        TransKey::Add => "追加".to_string(),
        TransKey::DeleteCompleted => "完了済みのタスクを削除".to_string(),
        TransKey::CompletedHeader => "完了済み".to_string(),
        TransKey::NoCompletedTasks => "削除する完了済みタスクがありません".to_string(),
        TransKey::TaskAdded => "タスクを追加しました".to_string(),
        TransKey::TaskCompleted => "タスクが完了しました！🎉".to_string(),
        TransKey::TaskUncompleted => "タスクを未完了にしました".to_string(),
        TransKey::TaskDeleted => "タスクを削除しました".to_string(),
        TransKey::TaskUpdated => "タスクを更新しました".to_string(),
        TransKey::NewListAdded => "新しいリストを追加しました".to_string(),
        TransKey::ListRenamed => "リスト名を変更しました".to_string(),
        TransKey::ListDeleted => "リストを削除しました".to_string(),
        TransKey::RenameCurrentList => "現在のリスト名を変更".to_string(),
        TransKey::AddNewList => "新しいリストを追加".to_string(),
        TransKey::HideLists => "リストを非表示".to_string(),
        TransKey::ManageLists => "リストを管理".to_string(),
        TransKey::ConfirmDeleteTask(t) => format!("「{}」を削除してもよろしいですか？", t),
        TransKey::ConfirmDeleteCompleted(n) => format!("完了済みのタスクを {} 件削除しますか？", n),
        TransKey::ConfirmDeleteList(l) => format!(
            "リスト「{}」とそこに含まれるすべてのタスクを削除しますか？",
            l
        ),
        TransKey::PromptRenameList => "新しいリスト名を入力してください：".to_string(),
        TransKey::ThemeToggle => "テーマ切り替え".to_string(),
        TransKey::ClearedCompleted(n) => format!("完了済みのタスクを {} 件削除しました", n),
    }
}
