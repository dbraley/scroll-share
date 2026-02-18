use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
        };

        tracing::error!(%status, error = %self, "request error");

        let body = axum::Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
