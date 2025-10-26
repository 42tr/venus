use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs::File, path::Path};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Extract the database file path from the URL
        if let Some(db_path) = database_url.strip_prefix("sqlite:") {
            // Create parent directory if it doesn't exist
            if let Some(parent) = Path::new(db_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Create the database file if it doesn't exist
            if !Path::new(db_path).exists() {
                File::create(db_path)?;
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}