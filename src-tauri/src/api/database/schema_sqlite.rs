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

        -- Sensor Types (dynamic, not hardcoded)
        CREATE TABLE IF NOT EXISTS sensor_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,           -- "DHT11", "BME280", "DS18B20"
            name TEXT NOT NULL,                  -- "DHT11 Temperature & Humidity"
            description TEXT,
            default_scale TEXT,                  -- JSON: [["temperature","C"],["humidity","%"]]
            is_active BOOLEAN DEFAULT TRUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Actuator Types (dynamic, not hardcoded)
        CREATE TABLE IF NOT EXISTS actuator_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,           -- "relay", "motor", "led"
            name TEXT NOT NULL,                  -- "Relay Switch"
            description TEXT,
            supported_commands TEXT,             -- JSON: ["ON","OFF"] or ["0-100"]
            is_active BOOLEAN DEFAULT TRUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS icons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            iconify_id TEXT NOT NULL,
            category TEXT NOT NULL,
            color TEXT,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_icons_category ON icons(category);
        CREATE INDEX IF NOT EXISTS idx_icons_active ON icons(is_active);

        CREATE TRIGGER IF NOT EXISTS trg_icons_updated_at
        AFTER UPDATE ON icons
        FOR EACH ROW
        BEGIN
            UPDATE icons SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
        END;

        -- Initial icons (sensors + actuators)
        INSERT OR IGNORE INTO icons (uuid, code, name, iconify_id, category, color) VALUES
            (lower(hex(randomblob(16))), 'thermometer', 'Temperature', 'mdi:thermometer', 'sensor', '#E53935'),
            (lower(hex(randomblob(16))), 'droplets', 'Humidity', 'lucide:droplets', 'sensor', '#1E88E5'),
            (lower(hex(randomblob(16))), 'wind', 'Air', 'mdi:wind-turbine', 'sensor', '#43A047'),
            (lower(hex(randomblob(16))), 'flame', 'Gas', 'mdi:fire', 'sensor', '#FB8C00'),
            (lower(hex(randomblob(16))), 'gauge', 'Pressure', 'mdi:gauge', 'sensor', '#8E24AA'),
            (lower(hex(randomblob(16))), 'sun', 'Light', 'mdi:white-balance-sunny', 'sensor', '#FDD835'),
            (lower(hex(randomblob(16))), 'activity', 'Monitoring', 'lucide:activity', 'sensor', '#00ACC1'),
            (lower(hex(randomblob(16))), 'bar-chart', 'Data', 'mdi:chart-bar', 'sensor', '#5C6BC0'),
            (lower(hex(randomblob(16))), 'power', 'On/Off', 'mdi:power', 'actuator', '#43A047'),
            (lower(hex(randomblob(16))), 'toggle', 'Switch', 'mdi:toggle-switch', 'actuator', '#1E88E5'),
            (lower(hex(randomblob(16))), 'robot', 'Robot', 'mdi:robot', 'actuator', '#7B1FA2'),
            (lower(hex(randomblob(16))), 'lamp', 'Lamp', 'mdi:lightbulb', 'actuator', '#FDD835'),
            (lower(hex(randomblob(16))), 'coil', 'Coil/Relay', 'mdi:flash', 'actuator', '#FB8C00'),
            (lower(hex(randomblob(16))), 'fan', 'Fan', 'mdi:fan', 'actuator', '#26A69A'),
            (lower(hex(randomblob(16))), 'lock', 'Lock', 'mdi:lock', 'actuator', '#5C6BC0'),
            (lower(hex(randomblob(16))), 'cpu', 'Generic', 'mdi:cpu-64-bit', 'actuator', '#78909C');

        -- Initial sensor types
        INSERT OR IGNORE INTO sensor_types (code, name, description, default_scale) VALUES
            ('DHT11', 'DHT11 Temperature & Humidity', 'Low-cost digital temperature and humidity sensor', '[["temperature","C"],["humidity","%"]]'),
            ('DHT22', 'DHT22 Temperature & Humidity', 'More accurate version of DHT11', '[["temperature","C"],["humidity","%"]]');

        -- Initial actuator types
        INSERT OR IGNORE INTO actuator_types (code, name, description, supported_commands) VALUES
            ('RELAY', 'Relay Switch', 'Simple on/off relay module', '["ON","OFF"]'),
            ('MAGNETIC', 'Magnetic Door/Window Sensor', 'Magnetic contact sensor for doors and windows', '["OPEN","CLOSED"]');

        CREATE TABLE IF NOT EXISTS devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            location_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            device_type TEXT NOT NULL CHECK(device_type IN ('actuator', 'sensor')),
            model TEXT NOT NULL,                    -- Board type: "ESP32", "RP2", "Pyboard"
            firmware_version TEXT,
            mac_address TEXT NOT NULL,
            sensor_type TEXT,                       -- For sensors: "DHT11", "BME280", "DS18B20", etc
            actuator_type TEXT,                     -- For actuators: "relay", "motor", "led", etc
            device_scale TEXT,
            parameter_ranges TEXT,                  -- Sensor: JSON { "measurement": { "unit", "min_reading", "max_reading" } } (ROLE.md)
            command_spec TEXT,                      -- Actuator: JSON { "type": "discrete"|"range", ... } (ROLE.md)
            adopted_at TEXT DEFAULT CURRENT_TIMESTAMP,
            operation_status TEXT DEFAULT 'offline' CHECK(operation_status IN ('online', 'offline')),
            last_seen_at TEXT,
            ip_address TEXT,
            publish_qos INTEGER DEFAULT 1 CHECK(publish_qos IN (0, 1, 2)),
            subscribe_qos INTEGER DEFAULT 1 CHECK(subscribe_qos IN (0, 1, 2)),
            status_retain BOOLEAN DEFAULT TRUE,
            data_retain BOOLEAN DEFAULT FALSE,
            lwt_enabled BOOLEAN DEFAULT TRUE,
            lwt_message TEXT DEFAULT '{"state":"offline","reason":"unexpected"}',
            lwt_qos INTEGER DEFAULT 1 CHECK(lwt_qos IN (0, 1, 2)),
            lwt_retain BOOLEAN DEFAULT TRUE,
            heartbeat_interval INTEGER DEFAULT 60,
            offline_threshold INTEGER DEFAULT 300,
            last_command TEXT,
            last_command_at TEXT,
            is_active BOOLEAN DEFAULT TRUE,
            icon_id INTEGER REFERENCES icons(id),
            position_x REAL,
            position_y REAL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(location_id) REFERENCES locations(id) ON DELETE CASCADE,
            UNIQUE(user_id, name),
            UNIQUE(mac_address)
        );

        CREATE INDEX IF NOT EXISTS idx_devices_user_id ON devices(user_id);
        CREATE INDEX IF NOT EXISTS idx_devices_location_id ON devices(location_id);
        CREATE INDEX IF NOT EXISTS idx_devices_uuid ON devices(uuid);
        CREATE INDEX IF NOT EXISTS idx_devices_operation_status ON devices(operation_status);
        CREATE INDEX IF NOT EXISTS idx_devices_user_location ON devices(user_id, location_id);
        CREATE INDEX IF NOT EXISTS idx_devices_mac ON devices(mac_address);

        CREATE TRIGGER IF NOT EXISTS trg_devices_updated_at
        AFTER UPDATE ON devices
        FOR EACH ROW
        BEGIN
            UPDATE devices SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
        END;

        CREATE TABLE IF NOT EXISTS device_commands (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            command TEXT NOT NULL,
            source TEXT DEFAULT 'user',
            sent_at TEXT DEFAULT CURRENT_TIMESTAMP,
            ack_at TEXT,
            response_ms INTEGER,
            FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_device_commands_device ON device_commands(device_id);
        CREATE INDEX IF NOT EXISTS idx_device_commands_sent ON device_commands(sent_at);
        CREATE INDEX IF NOT EXISTS idx_device_commands_device_sent ON device_commands(device_id, sent_at);

        CREATE TABLE IF NOT EXISTS sensor_readings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            measurement TEXT NOT NULL,           -- "temperature", "humidity", "pressure"
            value REAL NOT NULL,                 -- 25.5, 60.2, 1013.25
            scale TEXT NOT NULL,                 -- "C", "%", "hPa"
            recorded_at TEXT NOT NULL,           -- ISO8601 timestamp from device
            received_at TEXT DEFAULT CURRENT_TIMESTAMP,
            
            FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_readings_device_time 
            ON sensor_readings(device_id, recorded_at);
        
        CREATE INDEX IF NOT EXISTS idx_readings_device_measurement_time 
            ON sensor_readings(device_id, measurement, recorded_at);
        
        CREATE INDEX IF NOT EXISTS idx_readings_time 
            ON sensor_readings(recorded_at);

        CREATE TABLE IF NOT EXISTS triggers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            device_id INTEGER,
            name TEXT NOT NULL,
            source_event TEXT NOT NULL CHECK(source_event IN ('sensor_reading', 'device_command', 'schedule')),
            condition_json TEXT NOT NULL,
            action_type TEXT NOT NULL CHECK(action_type IN ('discord', 'telegram', 'device_command')),
            action_config_json TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE
        );

        CREATE TRIGGER IF NOT EXISTS trg_triggers_updated_at
        AFTER UPDATE ON triggers
        FOR EACH ROW
        BEGIN
            UPDATE triggers SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
        END;

        CREATE INDEX IF NOT EXISTS idx_triggers_user_id ON triggers(user_id);
        CREATE INDEX IF NOT EXISTS idx_triggers_device_id ON triggers(device_id);
        CREATE INDEX IF NOT EXISTS idx_triggers_uuid ON triggers(uuid);
        CREATE INDEX IF NOT EXISTS idx_triggers_user_active ON triggers(user_id, is_active);
        "#,
    )
        .execute(pool)
        .await?;

    // Migration: add icon_id to devices if missing (SQLite doesn't support ADD COLUMN IF NOT EXISTS)
    let _ = sqlx::query("ALTER TABLE devices ADD COLUMN icon_id INTEGER REFERENCES icons(id)")
        .execute(pool)
        .await;

    // Migration: add position_x, position_y for device placement on location image (Task_Device_Position_In_Location)
    let _ = sqlx::query("ALTER TABLE devices ADD COLUMN position_x REAL").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE devices ADD COLUMN position_y REAL").execute(pool).await;

    // Migration: parameter_ranges (sensor) and command_spec (actuator) — Task_Device_Types_Sensor_Actuator_Ranges
    let _ = sqlx::query("ALTER TABLE devices ADD COLUMN parameter_ranges TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE devices ADD COLUMN command_spec TEXT").execute(pool).await;

    Ok(())
}