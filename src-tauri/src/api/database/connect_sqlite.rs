use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite
};
use log::error;
use std::{path::Path, str::FromStr, time::Duration};

use crate::api::database::schema_sqlite::init_sqlite_schema;

struct SqliteConfig {
    database_path: String,
    max_connections: u32,
}

impl SqliteConfig{
    fn init_sqlite_config() -> SqliteConfig{
        SqliteConfig{
            database_path: std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "../database.db".to_string()),
            max_connections: 5
        }

    }

    pub fn get_database_path(&self) -> &str {
        &self.database_path
    }

    pub fn get_max_connections(&self) -> u32 {
        self.max_connections
    }
}

pub async fn get_sqlite_pool() -> Pool<Sqlite>{
    let config_sqlite = SqliteConfig::init_sqlite_config();
    let database_path = config_sqlite.get_database_path();
    let max_connections = config_sqlite.get_max_connections();
    
    if let Some(parent) = Path::new(database_path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("💥 Failed to create database directory: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    let connection_string = if database_path.starts_with("sqlite://") {
        database_path.to_string()
    } else {
        format!("sqlite://{}", database_path)
    };

    let pool = match SqlitePoolOptions::new()
        .max_connections(max_connections)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect_with(
            SqliteConnectOptions::from_str(&connection_string)
                .expect("Invalid SQLite connection string")
                .create_if_missing(true)
                //WAL + sync
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(5))
                .foreign_keys(true),
        )
        .await{
        Ok(pool) => pool,
        Err(e) => {
            error!("💥 Failed to create database directory: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = init_sqlite_schema(&pool).await {
        error!("💥 Failed to initialize schema: {:?}", e);
        std::process::exit(1);
    }

    pool

}