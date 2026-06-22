use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToastType {
    Success,
    Error,
}

impl Toast {
    // Constructor to quickly instantiate success or error toasts
    pub fn new(id: usize, message: String, toast_type: ToastType) -> Self {
        Self {
            id,
            message,
            toast_type,
        }
    }
}
