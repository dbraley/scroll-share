use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

use crate::routes::{auth, health};

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/auth/register", post(auth::register))
        .with_state(pool)
}
