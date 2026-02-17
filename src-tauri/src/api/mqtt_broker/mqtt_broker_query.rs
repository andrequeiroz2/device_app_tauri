use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::error::map_mqtt_broker_db_error;
use crate::api::mqtt_broker::mqtt_broker_model::{
    MqttBroker, MqttBrokerCreateDB, MqttBrokerFilter, MqttBrokerListResponse, MqttBrokerPublic,
};

#[instrument(skip(broker, pool), fields(uuid = %broker.uuid, user_id = broker.user_id, name = %broker.name, host = %broker.host, port = broker.port))]
pub async fn mqtt_broker_post_query(
    broker: &MqttBrokerCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<MqttBroker, String> {
    let rec = sqlx::query_as::<_, MqttBroker>(
        r#"
        INSERT INTO mqtt_brokers (
            uuid,
            user_id,
            name,
            description,
            host,
            port,
            username,
            password,
            use_tls,
            ca_certificate_path,
            client_certificate_path,
            client_key_path,
            insecure_tls,
            client_id,
            keep_alive_interval,
            clean_session,
            connection_timeout_secs,
            operation_timeout_secs,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain,
            is_active,
            is_connected,
            is_default
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)
        RETURNING
            id,
            uuid,
            user_id,
            name,
            description,
            host,
            port,
            username,
            password,
            use_tls,
            ca_certificate_path,
            client_certificate_path,
            client_key_path,
            insecure_tls,
            client_id,
            keep_alive_interval,
            clean_session,
            connection_timeout_secs,
            operation_timeout_secs,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain,
            is_active,
            is_connected,
            is_default,
            last_connected_at,
            last_connection_error,
            created_at,
            updated_at
        "#,
    )
    .bind(&broker.uuid)
    .bind(broker.user_id)
    .bind(&broker.name)
    .bind(&broker.description)
    .bind(&broker.host)
    .bind(broker.port)
    .bind(&broker.username)
    .bind(&broker.password)
    .bind(broker.use_tls)
    .bind(&broker.ca_certificate_path)
    .bind(&broker.client_certificate_path)
    .bind(&broker.client_key_path)
    .bind(broker.insecure_tls)
    .bind(&broker.client_id)
    .bind(broker.keep_alive_interval)
    .bind(broker.clean_session)
    .bind(broker.connection_timeout_secs)
    .bind(broker.operation_timeout_secs)
    .bind(&broker.last_will_topic)
    .bind(&broker.last_will_message)
    .bind(broker.last_will_qos)
    .bind(broker.last_will_retain)
    .bind(true) // is_active = true por padrão
    .bind(false) // is_connected = false por padrão
    .bind(broker.is_default)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: mqtt_broker_post_query");
        map_mqtt_broker_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id, exclude_uuid = exclude_uuid))]
pub async fn mqtt_broker_unset_other_defaults(
    user_id: i64,
    exclude_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE mqtt_brokers
        SET is_default = 0
        WHERE user_id = ?1 AND uuid != ?2 AND is_default = 1
        "#,
    )
    .bind(user_id)
    .bind(exclude_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: mqtt_broker_unset_other_defaults");
        map_mqtt_broker_db_error(&e)
    })?;

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, page = page, page_size = page_size, filter = ?filter))]
pub async fn mqtt_broker_list_query(
    user_id: i64,
    page: u32,
    page_size: u32,
    filter: &MqttBrokerFilter,
    pool: &Pool<Sqlite>,
) -> Result<MqttBrokerListResponse, String> {

    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    // Build WHERE conditions dynamically
    let mut where_clauses = vec!["user_id = ?".to_string()];
    
    // Add status filter
    let is_active_condition = if filter.show_all() {
        "1=1"
    } else {
        "is_active = 1"
    };
    where_clauses.push(is_active_condition.to_string());

    // Filter by name (LIKE, case-insensitive)
    if let Some(ref name) = filter.name {
        if !name.trim().is_empty() {
            where_clauses.push("LOWER(name) LIKE LOWER(?)".to_string());
        }
    }

    // Filter by port
    if filter.port.is_some() {
        where_clauses.push("port = ?".to_string());
    }

    // Filter by default
    if filter.default.is_some() {
        where_clauses.push("is_default = ?".to_string());
    }

    // Filter by connected
    if filter.connected.is_some() {
        where_clauses.push("is_connected = ?".to_string());
    }

    let where_clause = where_clauses.join(" AND ");
    let query_str = format!(
        r#"
        SELECT
            id, uuid, user_id, name, description, host, port,
            username, password, use_tls, ca_certificate_path,
            client_certificate_path, client_key_path, insecure_tls,
            client_id, keep_alive_interval, clean_session,
            connection_timeout_secs, operation_timeout_secs,
            last_will_topic, last_will_message, last_will_qos,
            last_will_retain, is_active, is_connected, is_default,
            last_connected_at, last_connection_error,
            created_at, updated_at
        FROM mqtt_brokers
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_clause
    );

    let mut query = sqlx::query_as::<_, MqttBroker>(&query_str)
        .bind(user_id);

    // Bind filter parameters in order
    if let Some(ref name) = filter.name {
        if !name.trim().is_empty() {
            query = query.bind(format!("%{}%", name.trim()));
        }
    }

    if let Some(port) = filter.port {
        query = query.bind(port as i64);
    }

    if let Some(is_default) = filter.default {
        query = query.bind(is_default);
    }

    if let Some(is_connected) = filter.connected {
        query = query.bind(is_connected);
    }

    let items = query
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: mqtt_broker_list_query");
            map_mqtt_broker_db_error(&e)
        })?;

    let total = mqtt_broker_count_query(user_id, filter, pool).await?;

    let public_items: Vec<MqttBrokerPublic> = items
        .into_iter()
        .map(|broker| MqttBrokerPublic::from(broker))
        .collect();

    Ok(MqttBrokerListResponse {
        items: public_items,
        total,
        page,
        page_size,
    })
}

#[instrument(skip(pool), fields(user_id = user_id, filter = ?filter))]
pub async fn mqtt_broker_count_query(
    user_id: i64,
    filter: &MqttBrokerFilter,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    // Build WHERE conditions dynamically (same as list query)
    let mut where_clauses = vec!["user_id = ?".to_string()];
    
    // Add status filter
    let is_active_condition = if filter.show_all() {
        "1=1"
    } else {
        "is_active = 1"
    };
    where_clauses.push(is_active_condition.to_string());

    // Filter by name (LIKE, case-insensitive)
    if let Some(ref name) = filter.name {
        if !name.trim().is_empty() {
            where_clauses.push("LOWER(name) LIKE LOWER(?)".to_string());
        }
    }

    // Filter by port
    if let Some(_) = filter.port {
        where_clauses.push("port = ?".to_string());
    }

    // Filter by default
    if let Some(_) = filter.default {
        where_clauses.push("is_default = ?".to_string());
    }

    // Filter by connected
    if let Some(_) = filter.connected {
        where_clauses.push("is_connected = ?".to_string());
    }

    let where_clause = where_clauses.join(" AND ");
    let query_str = format!("SELECT COUNT(*) as total FROM mqtt_brokers WHERE {}", where_clause);

    let mut query = sqlx::query_as::<_, (i64,)>(&query_str)
        .bind(user_id);

    // Bind filter parameters in order (same as list query)
    if let Some(ref name) = filter.name {
        if !name.trim().is_empty() {
            query = query.bind(format!("%{}%", name.trim()));
        }
    }

    if let Some(port) = filter.port {
        query = query.bind(port as i64);
    }

    if let Some(is_default) = filter.default {
        query = query.bind(is_default);
    }

    if let Some(is_connected) = filter.connected {
        query = query.bind(is_connected);
    }

    let (total,): (i64,) = query
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: mqtt_broker_count_query");
            map_mqtt_broker_db_error(&e)
        })?;

    Ok(total)
}

#[instrument(skip(pool), fields(user_id = user_id, broker_uuid = broker_uuid))]
pub async fn mqtt_broker_soft_delete_query(
    user_id: i64,
    broker_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE mqtt_brokers
        SET is_active = 0
        WHERE user_id = ?1 AND uuid = ?2 AND is_active = 1
        "#,
    )
    .bind(user_id)
    .bind(broker_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: mqtt_broker_soft_delete_query");
        map_mqtt_broker_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Broker not found".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, broker_uuid = broker_uuid))]
pub async fn mqtt_broker_get_by_uuid_query(
    user_id: i64,
    broker_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<MqttBroker, String> {
    use sqlx::Error as SqlxError;

    let rec = sqlx::query_as::<_, MqttBroker>(
        r#"
        SELECT
            id, uuid, user_id, name, description, host, port,
            username, password, use_tls, ca_certificate_path,
            client_certificate_path, client_key_path, insecure_tls,
            client_id, keep_alive_interval, clean_session,
            connection_timeout_secs, operation_timeout_secs,
            last_will_topic, last_will_message, last_will_qos,
            last_will_retain, is_active, is_connected, is_default,
            last_connected_at, last_connection_error,
            created_at, updated_at
        FROM mqtt_brokers
        WHERE uuid = ?1 AND user_id = ?2
        "#,
    )
    .bind(broker_uuid)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Broker not found".to_string();
        }
        error!(error = %e, "fn: mqtt_broker_get_by_uuid_query");
        map_mqtt_broker_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id, broker_uuid = broker_uuid))]
pub async fn mqtt_broker_update_query(
    user_id: i64,
    broker_uuid: &str,
    is_active: Option<bool>,
    pool: &Pool<Sqlite>,
) -> Result<MqttBroker, String> {
    // Build dynamic UPDATE query
    let mut set_clauses = Vec::new();
    
    if is_active.is_some() {
        set_clauses.push("is_active = ?".to_string());
    }

    if set_clauses.is_empty() {
        return Err("No fields to update".to_string());
    }

    let set_clause = set_clauses.join(", ");

    let query_str = format!(
        r#"
        UPDATE mqtt_brokers
        SET {}
        WHERE uuid = ? AND user_id = ?
        RETURNING
            id, uuid, user_id, name, description, host, port,
            username, password, use_tls, ca_certificate_path,
            client_certificate_path, client_key_path, insecure_tls,
            client_id, keep_alive_interval, clean_session,
            connection_timeout_secs, operation_timeout_secs,
            last_will_topic, last_will_message, last_will_qos,
            last_will_retain, is_active, is_connected, is_default,
            last_connected_at, last_connection_error,
            created_at, updated_at
        "#,
        set_clause
    );

    let mut query = sqlx::query_as::<_, MqttBroker>(&query_str);

    if let Some(active) = is_active {
        query = query.bind(active);
    }

    let rec = query
        .bind(broker_uuid)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                return "Broker not found".to_string();
            }
            error!(error = %e, "fn: mqtt_broker_update_query");
            map_mqtt_broker_db_error(&e)
        })?;

    Ok(rec)
}

