mod common;

use common::spawn_app;
use serde_json::json;
use uuid::Uuid;

fn unique(prefix: &str) -> String {
    format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
}

async fn register_user(app: &common::TestApp, username: &str, password: &str) {
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": username,
            "display_name": "Test User",
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send register request");
    assert_eq!(response.status(), 201, "Registration should succeed");
}

#[tokio::test]
async fn login_with_valid_credentials_returns_access_and_refresh_tokens() {
    let app = spawn_app().await;
    let username = unique("login");
    let password = "securepassword123";

    register_user(&app, &username, password).await;

    let response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["access_token"].is_string(), "should return access_token");
    assert!(body["refresh_token"].is_string(), "should return refresh_token");
    assert_eq!(body["token_type"], "Bearer");
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let app = spawn_app().await;
    let username = unique("badpw");
    register_user(&app, &username, "securepassword123").await;

    let response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "username": username,
            "password": "wrongpassword"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn login_with_nonexistent_user_returns_401() {
    let app = spawn_app().await;

    let response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "username": "nosuchuser_ever",
            "password": "somepassword123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Same 401 as wrong password — no user enumeration
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn access_token_is_valid_rs256_jwt_with_correct_claims() {
    let app = spawn_app().await;
    let username = unique("claims");
    register_user(&app, &username, "securepassword123").await;

    let login_body: serde_json::Value = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "username": username, "password": "securepassword123" }))
        .send()
        .await
        .expect("Failed to send login request")
        .json()
        .await
        .expect("Failed to parse login response");

    let access_token = login_body["access_token"].as_str().unwrap();

    // Decode without verification to inspect claims
    let parts: Vec<&str> = access_token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts");

    // Decode the payload (part 1)
    use base64::Engine;
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .expect("Failed to decode JWT payload");
    let claims: serde_json::Value =
        serde_json::from_slice(&payload_bytes).expect("Failed to parse JWT claims");

    assert_eq!(claims["iss"], "auth-service");
    assert!(claims["sub"].is_string(), "should have sub claim");
    assert_eq!(claims["username"], username);
    assert!(claims["exp"].is_number(), "should have exp claim");
    assert!(claims["iat"].is_number(), "should have iat claim");

    // Access token should expire in roughly 15 minutes (allow 14-16 min window)
    let exp = claims["exp"].as_i64().unwrap();
    let iat = claims["iat"].as_i64().unwrap();
    let ttl = exp - iat;
    assert!(ttl >= 840 && ttl <= 960, "TTL should be ~900s (15min), got {ttl}s");
}

#[tokio::test]
async fn refresh_with_valid_token_returns_new_token_pair() {
    let app = spawn_app().await;
    let username = unique("refresh");
    register_user(&app, &username, "securepassword123").await;

    let login_body: serde_json::Value = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "username": username, "password": "securepassword123" }))
        .send()
        .await
        .expect("Failed to send login request")
        .json()
        .await
        .expect("Failed to parse login response");

    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    let response = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .expect("Failed to send refresh request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["access_token"].is_string(), "should return new access_token");
    assert!(body["refresh_token"].is_string(), "should return new refresh_token");

    // New refresh token should be different from the old one (rotation)
    assert_ne!(body["refresh_token"].as_str().unwrap(), refresh_token);
}

#[tokio::test]
async fn refresh_with_used_token_returns_401() {
    let app = spawn_app().await;
    let username = unique("usedtok");
    register_user(&app, &username, "securepassword123").await;

    let login_body: serde_json::Value = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "username": username, "password": "securepassword123" }))
        .send()
        .await
        .expect("Failed to send login request")
        .json()
        .await
        .expect("Failed to parse login response");

    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Use the refresh token once (should succeed)
    let response = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .expect("Failed to send refresh request");
    assert_eq!(response.status(), 200);

    // Use the same refresh token again (should fail — it was rotated/revoked)
    let response = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .expect("Failed to send refresh request");
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn refresh_with_invalid_token_returns_401() {
    let app = spawn_app().await;

    let response = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": "totally-bogus-token" }))
        .send()
        .await
        .expect("Failed to send refresh request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn jwks_endpoint_returns_rsa_public_key() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(app.url("/.well-known/jwks.json"))
        .send()
        .await
        .expect("Failed to send JWKS request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let keys = body["keys"].as_array().expect("keys should be an array");
    assert_eq!(keys.len(), 1);

    let key = &keys[0];
    assert_eq!(key["kty"], "RSA");
    assert_eq!(key["alg"], "RS256");
    assert_eq!(key["use"], "sig");
    assert!(key["n"].is_string(), "should have modulus");
    assert!(key["e"].is_string(), "should have exponent");
    assert!(key["kid"].is_string(), "should have key id");
}

#[tokio::test]
async fn openid_configuration_returns_discovery_document() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(app.url("/.well-known/openid-configuration"))
        .send()
        .await
        .expect("Failed to send discovery request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["issuer"].is_string());
    assert!(body["jwks_uri"].is_string());
    assert!(body["token_endpoint"].is_string());
}

#[tokio::test]
async fn roundtrip_login_then_verify_jwt_with_jwks() {
    let app = spawn_app().await;
    let username = unique("roundtrip");
    register_user(&app, &username, "securepassword123").await;

    // Login
    let login_body: serde_json::Value = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "username": username, "password": "securepassword123" }))
        .send()
        .await
        .expect("Failed to send login request")
        .json()
        .await
        .expect("Failed to parse login response");

    let access_token = login_body["access_token"].as_str().unwrap();

    // Fetch JWKS
    let jwks_body: serde_json::Value = app
        .client
        .get(app.url("/.well-known/jwks.json"))
        .send()
        .await
        .expect("Failed to send JWKS request")
        .json()
        .await
        .expect("Failed to parse JWKS response");

    let key = &jwks_body["keys"][0];
    let n = key["n"].as_str().unwrap();
    let e = key["e"].as_str().unwrap();

    // Verify the JWT signature using the JWKS public key
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(n, e)
        .expect("Failed to create decoding key from JWKS");

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&["auth-service"]);
    validation.validate_aud = false;

    let token_data = jsonwebtoken::decode::<serde_json::Value>(
        access_token,
        &decoding_key,
        &validation,
    )
    .expect("JWT verification failed — signature does not match JWKS public key");

    assert_eq!(token_data.claims["username"], username);
}
