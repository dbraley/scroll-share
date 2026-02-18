mod common;

use common::spawn_app;

#[tokio::test]
async fn healthz_returns_200_with_status_ok() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(app.url("/healthz"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn readyz_returns_200_when_db_is_reachable() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(app.url("/readyz"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn readyz_returns_503_when_db_is_unreachable() {
    // Create a pool with a lazy connection to an unreachable database.
    // acquire_timeout keeps the test fast — the query fails quickly instead of
    // waiting for the default TCP connect timeout.
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_lazy("postgresql://invalid:invalid@localhost:59999/nonexistent")
        .unwrap();

    let app = auth_service::app::create_router(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/readyz", addr))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 503);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "unavailable");
}
