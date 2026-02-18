use auth_service::app::create_router;
use auth_service::config::Config;
use auth_service::db::create_pool;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();
    tracing::info!("Starting auth-service on {}:{}", config.host, config.port);

    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = create_router(pool);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
