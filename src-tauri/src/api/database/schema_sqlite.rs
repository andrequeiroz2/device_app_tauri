use sqlx::{query, Pool, Sqlite};

pub async fn init_sqlite_schema(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            username TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TRIGGER IF NOT EXISTS trg_users_updated_at
        AFTER UPDATE ON users
        FOR EACH ROW
        BEGIN
            UPDATE users
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = OLD.id;
        END;
        "#,
    )
        .execute(pool)
        .await?;

    Ok(())
}