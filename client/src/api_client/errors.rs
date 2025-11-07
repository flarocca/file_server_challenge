use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("File not found")]
    NotFound,

    #[error("File already exists")]
    Conflict,

    #[error("Authentication failed or invalid signature")]
    Unauthorized,

    #[error("Unexpected Server response: {0}")]
    Other(StatusCode, String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl ApiClientError {
    fn new(status: StatusCode, message: &str) -> Self {
        match status {
            StatusCode::NOT_FOUND => ApiClientError::NotFound,
            StatusCode::CONFLICT => ApiClientError::Conflict,
            StatusCode::UNAUTHORIZED => ApiClientError::Unauthorized,
            s if s.is_client_error() => ApiClientError::Other(status, message.to_owned()),
            _ => ApiClientError::Unexpected(format!("{message} (status: {status})")),
        }
    }

    pub async fn from_response(resp: reqwest::Response) -> Self {
        let status = resp.status();
        let message = resp.text().await.unwrap_or_default();

        ApiClientError::new(status, &message)
    }
}

impl From<reqwest::Error> for ApiClientError {
    fn from(err: reqwest::Error) -> Self {
        if let Some(status) = err.status() {
            let message = err.to_string();

            ApiClientError::new(status, &message)
        } else {
            ApiClientError::Unexpected(err.to_string())
        }
    }
}
