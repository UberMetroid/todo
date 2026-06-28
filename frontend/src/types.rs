use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToastType {
    Success,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
}
