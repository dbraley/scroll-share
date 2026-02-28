use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::user::{CreateUserRequest, UserResponse};
use crate::services::password::hash_password;

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()
        .map_err(|errors| AppError::Validation(errors.join("; ")))?;

    let password_hash =
        hash_password(&req.password).map_err(|e| AppError::Internal(e.to_string()))?;

    let user = sqlx::query_as::<_, crate::models::user::User>(
        "INSERT INTO users (username, display_name, password_hash) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&req.username)
    .bind(&req.display_name)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_username_key") => {
            AppError::Conflict("username already taken".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}
