use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

use crate::routes::health;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .with_state(pool)
}
