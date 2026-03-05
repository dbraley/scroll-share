use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;

use crate::app::AppState;

pub async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

// Used by the test harness directly with a bare PgPool (for the 503 test)
pub async fn readyz(State(pool): State<PgPool>) -> impl IntoResponse {
    ping_db(&pool).await
}

// Used by the main router with full AppState
pub async fn readyz_with_pool(State(state): State<AppState>) -> impl IntoResponse {
    ping_db(&state.pool).await
}

async fn ping_db(pool: &PgPool) -> impl IntoResponse + use<> {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unavailable" })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn healthz_returns_ok() {
        let response = healthz().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
