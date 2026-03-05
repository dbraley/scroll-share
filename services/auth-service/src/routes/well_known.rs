use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::app::AppState;

pub async fn jwks(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.token_service.jwks().clone())
}

pub async fn openid_configuration(State(state): State<AppState>) -> impl IntoResponse {
    let issuer = "auth-service";
    let base = &state.base_url;
    Json(serde_json::json!({
        "issuer": issuer,
        "token_endpoint": format!("{}/auth/login", base),
        "jwks_uri": format!("{}/.well-known/jwks.json", base),
        "response_types_supported": ["token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    }))
}
