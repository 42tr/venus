use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub auth_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./venus.db".to_string());
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8085".to_string())
            .parse()
            .unwrap_or(8085);
        
        let auth_url = env::var("AUTH_URL")
            .unwrap_or_else(|_| "http://localhost:8080/auth".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());

        Ok(Self {
            database_url,
            port,
            auth_url,
            jwt_secret,
        })
    }
}