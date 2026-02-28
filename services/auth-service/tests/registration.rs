mod common;

use common::spawn_app;
use serde_json::json;
use uuid::Uuid;

fn unique(prefix: &str) -> String {
    format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
}

#[tokio::test]
async fn register_with_valid_data_returns_201_and_user_profile() {
    let app = spawn_app().await;
    let username = unique("newuser");

    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": username,
            "display_name": "New User",
            "password": "securepassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["username"], username);
    assert_eq!(body["display_name"], "New User");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    // password_hash must not be exposed
    assert!(body.get("password_hash").is_none());
}

#[tokio::test]
async fn register_duplicate_username_returns_409() {
    let app = spawn_app().await;
    let username = unique("dupeuser");

    let payload = json!({
        "username": username,
        "display_name": "First User",
        "password": "securepassword123"
    });

    // Register first user
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 201);

    // Register duplicate
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": username,
            "display_name": "Second User",
            "password": "anotherpassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 409);
}

#[tokio::test]
async fn register_with_invalid_fields_returns_422() {
    let app = spawn_app().await;

    // Short username
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": "ab",
            "display_name": "Test",
            "password": "securepassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 422);

    // Short password
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": "validuser",
            "display_name": "Test",
            "password": "short"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 422);

    // Empty display name
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": "validuser2",
            "display_name": "",
            "password": "securepassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 422);
}

#[tokio::test]
async fn register_stores_password_as_argon2id_hash() {
    let app = spawn_app().await;
    let username = unique("hashcheck");

    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": username,
            "display_name": "Hash Check",
            "password": "mypassword123"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 201);

    // Verify the hash in the database is Argon2id
    let row: (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&app.pool)
        .await
        .expect("Failed to query user");

    assert!(
        row.0.starts_with("$argon2id$"),
        "Password hash should be Argon2id, got: {}",
        &row.0[..20]
    );
}

#[tokio::test]
async fn register_with_missing_fields_returns_422() {
    let app = spawn_app().await;

    // Missing password field entirely
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "username": "testuser",
            "display_name": "Test"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 422);
}
