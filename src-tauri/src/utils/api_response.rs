use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccessResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiSuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data, message: None }
    }
}
