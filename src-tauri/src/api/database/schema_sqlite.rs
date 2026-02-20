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

        CREATE TABLE IF NOT EXISTS mqtt_brokers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 1883,
            username TEXT,
            password TEXT,
            use_tls BOOLEAN DEFAULT FALSE,
            ca_certificate_path TEXT,
            client_certificate_path TEXT,
            client_key_path TEXT,
            insecure_tls BOOLEAN DEFAULT FALSE,
            client_id TEXT,
            keep_alive_interval INTEGER DEFAULT 60,
            clean_session BOOLEAN DEFAULT TRUE,
            connection_timeout_secs INTEGER DEFAULT 30,
            operation_timeout_secs INTEGER DEFAULT 30,
            last_will_topic TEXT,
            last_will_message TEXT,
            last_will_qos INTEGER DEFAULT 0,
            last_will_retain BOOLEAN DEFAULT FALSE,
            is_active BOOLEAN DEFAULT TRUE,
            is_connected BOOLEAN DEFAULT FALSE,
            is_default BOOLEAN DEFAULT FALSE,
            last_connected_at TEXT,
            last_connection_error TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, name),
            UNIQUE(user_id, host, port),
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE RESTRICT ON UPDATE CASCADE,
            CHECK(port > 0 AND port <= 65535),
            CHECK(keep_alive_interval > 0),
            CHECK(last_will_qos IN (0, 1, 2))
        );

        CREATE TRIGGER IF NOT EXISTS trg_mqtt_brokers_updated_at
        AFTER UPDATE ON mqtt_brokers
        FOR EACH ROW
        BEGIN
            UPDATE mqtt_brokers
            SET updated_at = CURRENT_TIMESTAMP
            WHERE id = OLD.id;
        END;

        CREATE INDEX IF NOT EXISTS idx_mqtt_brokers_user_id ON mqtt_brokers(user_id);
        CREATE INDEX IF NOT EXISTS idx_mqtt_brokers_uuid ON mqtt_brokers(uuid);
        CREATE INDEX IF NOT EXISTS idx_mqtt_brokers_is_default ON mqtt_brokers(user_id, is_default);

        CREATE TABLE IF NOT EXISTS mqtt_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            topic TEXT NOT NULL,
            broker_uuid TEXT,
            device_uuid TEXT,
            payload TEXT NOT NULL,
            qos INTEGER DEFAULT 0,
            retain BOOLEAN DEFAULT FALSE,
            received_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_mqtt_messages_topic ON mqtt_messages(topic);
        CREATE INDEX IF NOT EXISTS idx_mqtt_messages_broker_uuid ON mqtt_messages(broker_uuid);
        CREATE INDEX IF NOT EXISTS idx_mqtt_messages_device_uuid ON mqtt_messages(device_uuid);
        CREATE INDEX IF NOT EXISTS idx_mqtt_messages_broker_device ON mqtt_messages(broker_uuid, device_uuid);
        CREATE INDEX IF NOT EXISTS idx_mqtt_messages_received_at ON mqtt_messages(received_at);

        CREATE TABLE IF NOT EXISTS password_reset_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,
            used_at TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token ON password_reset_tokens(token);
        CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
        CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
        CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_created ON password_reset_tokens(user_id, created_at);

        CREATE TABLE IF NOT EXISTS collector_notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            notification_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            broker_uuid TEXT,
            device_uuid TEXT,
            is_read BOOLEAN DEFAULT FALSE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_collector_notifications_user_created ON collector_notifications(user_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_collector_notifications_user_read ON collector_notifications(user_id, is_read);
        "#,
    )
        .execute(pool)
        .await?;

    Ok(())
}