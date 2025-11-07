use crate::services::FileServiceError;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct ServerError {
    status: StatusCode,
    body: Option<Value>,
}

impl ServerError {
    pub fn new(status: StatusCode, body: Option<Value>) -> Self {
        Self { status, body }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self.body {
            Some(v) => (self.status, Json(v)).into_response(),
            None => self.status.into_response(),
        }
    }
}

impl From<FileServiceError> for ServerError {
    fn from(err: FileServiceError) -> Self {
        match err {
            FileServiceError::FileNotFound => ServerError::new(
                StatusCode::NOT_FOUND,
                Some(json!({ "error": "File not found" })),
            ),
            FileServiceError::FileIndexNotFound => ServerError::new(
                StatusCode::NOT_FOUND,
                Some(json!({ "error": "File index not found" })),
            ),
            FileServiceError::FileAlreadyExists => ServerError::new(
                StatusCode::CONFLICT,
                Some(json!({ "error": "File already exists" })),
            ),
            FileServiceError::StorageError(msg) => ServerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({ "error": msg })),
            ),
        }
    }
}
