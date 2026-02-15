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

        CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            address TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            image_path TEXT,
            thumb_path TEXT,
            image_original_name TEXT,
            image_mime TEXT,
            image_size_bytes INTEGER,
            image_checksum_sha256 TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, name),
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE RESTRICT ON UPDATE CASCADE
        );

        CREATE TRIGGER IF NOT EXISTS trg_locations_updated_at
        AFTER UPDATE ON locations
        FOR EACH ROW
        BEGIN
            UPDATE locations
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = OLD.id;
        END;

        CREATE INDEX IF NOT EXISTS idx_locations_user_id ON locations(user_id);
        CREATE INDEX IF NOT EXISTS idx_locations_uuid ON locations(uuid);
        CREATE INDEX IF NOT EXISTS idx_locations_name ON locations(name);
        "#,
    )
        .execute(pool)
        .await?;

    Ok(())
}