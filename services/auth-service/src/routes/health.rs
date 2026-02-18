use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;

pub async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub async fn readyz(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
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
