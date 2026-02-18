use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://auth_user:auth_dev_password@localhost:15432/scrollshare?options=-csearch_path%3Dauth".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .expect("PORT must be a valid u16"),
        }
    }
}
