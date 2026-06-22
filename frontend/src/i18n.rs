use yew::prelude::*;

mod de;
mod en;
mod es;
mod fr;
mod ja;
mod pt;
mod ru;
mod zh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    En,
    Zh,
    Es,
    De,
    Ja,
    Fr,
    Pt,
    Ru,
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zh" => Self::Zh,
            "es" => Self::Es,
            "de" => Self::De,
            "ja" => Self::Ja,
            "fr" => Self::Fr,
            "pt" => Self::Pt,
            "ru" => Self::Ru,
            _ => Self::En,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
            Self::Es => "es",
            Self::De => "de",
            Self::Ja => "ja",
            Self::Fr => "fr",
            Self::Pt => "pt",
            Self::Ru => "ru",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::En => Self::Zh,
            Self::Zh => Self::Es,
            Self::Es => Self::De,
            Self::De => Self::Ja,
            Self::Ja => Self::Fr,
            Self::Fr => Self::Pt,
            Self::Pt => Self::Ru,
            Self::Ru => Self::En,
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
        Locale::En => en::translate(key),
        Locale::Zh => zh::translate(key),
        Locale::Es => es::translate(key),
        Locale::De => de::translate(key),
        Locale::Ja => ja::translate(key),
        Locale::Fr => fr::translate(key),
        Locale::Pt => pt::translate(key),
        Locale::Ru => ru::translate(key),
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
