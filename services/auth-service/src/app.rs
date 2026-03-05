use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

use crate::routes::{auth, health, well_known};
use crate::services::token::TokenService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub token_service: TokenService,
    pub base_url: String,
}

pub fn create_router(pool: PgPool) -> Router {
    let rsa_pem = std::env::var("RSA_PRIVATE_KEY_PEM").ok();
    let token_service = TokenService::new(rsa_pem.as_deref());
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());

    let state = AppState {
        pool,
        token_service,
        base_url,
    };

    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz_with_pool))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/.well-known/jwks.json", get(well_known::jwks))
        .route(
            "/.well-known/openid-configuration",
            get(well_known::openid_configuration),
        )
        .with_state(state)
}
