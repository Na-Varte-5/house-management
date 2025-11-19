use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad_request: {0}")]
    BadRequest(String),
    #[error("internal_error: {0}")]
    Internal(String),
    #[error("db_error: {0}")]
    Db(#[from] diesel::result::Error),
    #[error("crypto_error: {0}")]
    Crypto(String),
    #[error("token_error")]
    Token,
    #[error("not_found")]
    NotFound,
    #[error("attachment_too_large")]
    AttachmentTooLarge,
    #[error("invalid_mime_type")]
    InvalidMimeType,
    #[error("not_published")]
    NotPublished,
    #[error("expired")]
    Expired,
    #[error("comments_disabled")]
    CommentsDisabled,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized | AppError::Token => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::AttachmentTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::InvalidMimeType => StatusCode::BAD_REQUEST,
            AppError::NotPublished => StatusCode::NOT_FOUND, // hide drafts
            AppError::Expired => StatusCode::GONE,
            AppError::CommentsDisabled => StatusCode::FORBIDDEN,
            AppError::Db(_) | AppError::Internal(_) | AppError::Crypto(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({"error": self.to_string()}))
    }
}
