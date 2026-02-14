use serde::{Serialize};

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

}

#[derive(Serialize)]
pub struct ApiError {
    pub success: bool,
    pub message: String,
}

impl ApiError {
    pub fn err(message: String) -> Self {
        Self {
            success: false,
            message,
        }
    }
}

