use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use serde::Deserialize;

use crate::app::AppState;
use crate::errors::AppError;
use crate::models::user::{CreateUserRequest, UserResponse};
use crate::services::password::{hash_password, verify_password};
use crate::services::token::TokenService;

pub async fn register(
    State(state): State<AppState>,
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
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_username_key") => {
            AppError::Conflict("username already taken".to_string())
        }
        _ => AppError::Database(e),
    })?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch user — on not found, run a dummy verify to prevent timing attacks
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(&state.pool)
    .await?;

    let user = match user {
        Some(u) => {
            let valid = verify_password(&req.password, &u.password_hash)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if !valid {
                return Err(AppError::Unauthorized);
            }
            u
        }
        None => {
            // Dummy verify to equalise timing between nonexistent and wrong-password
            let _ = verify_password(&req.password, "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
            return Err(AppError::Unauthorized);
        }
    };

    let access_token = state
        .token_service
        .create_access_token(user.id, &user.username)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let (refresh_token, token_hash) = TokenService::generate_refresh_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_hash = TokenService::hash_refresh_token(&req.refresh_token);

    // Find a valid, non-revoked, non-expired token
    let row = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid)>(
        "SELECT id, user_id FROM refresh_tokens
         WHERE token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > now()",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await?;

    let (token_id, user_id) = row.ok_or(AppError::Unauthorized)?;

    // Revoke the used token (rotation)
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(&state.pool)
        .await?;

    // Fetch user for new token claims
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    let access_token = state
        .token_service
        .create_access_token(user.id, &user.username)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let (new_refresh_token, new_token_hash) = TokenService::generate_refresh_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&new_token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
    }))
}
