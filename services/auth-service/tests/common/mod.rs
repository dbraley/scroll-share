use auth_service::app::create_router;
use sqlx::PgPool;
use tokio::net::TcpListener;

pub struct TestApp {
    pub addr: String,
    pub pool: PgPool,
    pub client: reqwest::Client,
}

impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

pub async fn spawn_app() -> TestApp {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://auth_user:auth_dev_password@localhost:15432/scrollshare?options=-csearch_path%3Dauth".to_string()
    });

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = create_router(pool.clone());
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();

    TestApp { addr, pool, client }
}
