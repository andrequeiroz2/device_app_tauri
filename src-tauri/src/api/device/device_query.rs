use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::device::device_model::{
    Device, DeviceCommand, DeviceCommandChartPoint, DeviceCommandDailyStats, DeviceCommandFilter,
    DeviceCommandSummary, DeviceCreateDB, DeviceFilter, DeviceTypeFilter, DeviceUpdateDB,
    IsActiveFilter, OperationStatusFilter,
};
use crate::api::error::map_device_db_error;
use sqlx::Error as SqlxError;

#[instrument(skip(device, pool), fields(uuid = %device.uuid, user_id = device.user_id, name = %device.name, mac = %device.mac_address))]
pub async fn device_post_query(
    device: &DeviceCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<Device, String> {
    let rec = sqlx::query_as::<_, Device>(
        r#"
        INSERT INTO devices (
            uuid, user_id, location_id, name, description,
            device_type, model, mac_address, firmware_version,
            sensor_type, actuator_type, device_scale,
            adopted_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)
        RETURNING
            id, uuid, user_id, location_id, name, description, device_type,
            model, firmware_version, mac_address, sensor_type, actuator_type, device_scale,
            adopted_at, operation_status, last_seen_at, ip_address, publish_qos, subscribe_qos,
            status_retain, data_retain, lwt_enabled, lwt_message, lwt_qos, lwt_retain,
            heartbeat_interval, offline_threshold, last_command, last_command_at,
            is_active, created_at, updated_at
        "#,
    )
    .bind(&device.uuid)
    .bind(device.user_id)
    .bind(device.location_id)
    .bind(&device.name)
    .bind(&device.description)
    .bind(&device.device_type)
    .bind(&device.model)
    .bind(&device.mac_address)
    .bind(&device.firmware_version)
    .bind(&device.sensor_type)
    .bind(&device.actuator_type)
    .bind(&device.device_scale)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_post_query");
        map_device_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id, device_uuid = device_uuid))]
pub async fn device_soft_delete_query(
    user_id: i64,
    device_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE devices
        SET is_active = 0
        WHERE user_id = ?1 AND uuid = ?2 AND is_active = 1
        "#,
    )
    .bind(user_id)
    .bind(device_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_soft_delete_query");
        map_device_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Device not found".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, page = page, page_size = page_size, filter = ?filter))]
pub async fn device_list_query(
    user_id: i64,
    page: u32,
    page_size: u32,
    filter: &DeviceFilter,
    pool: &Pool<Sqlite>,
) -> Result<Vec<Device>, String> {
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    let mut conditions = vec!["user_id = ?1".to_string()];

    match filter.is_active {
        IsActiveFilter::Active => conditions.push("is_active = 1".to_string()),
        IsActiveFilter::Inactive => conditions.push("is_active = 0".to_string()),
        IsActiveFilter::All => {}
    }

    match filter.operation_status {
        OperationStatusFilter::Online => conditions.push("operation_status = 'online'".to_string()),
        OperationStatusFilter::Offline => conditions.push("operation_status = 'offline'".to_string()),
        OperationStatusFilter::All => {}
    }

    match filter.device_type {
        DeviceTypeFilter::Actuator => conditions.push("device_type = 'actuator'".to_string()),
        DeviceTypeFilter::Sensor => conditions.push("device_type = 'sensor'".to_string()),
        DeviceTypeFilter::All => {}
    }

    let location_bind = if filter.location_uuid.is_some() {
        conditions.push(
            "location_id = (SELECT id FROM locations WHERE uuid = ?4 AND user_id = ?1)".to_string()
        );
        true
    } else {
        false
    };

    let where_clause = conditions.join(" AND ");

    let query_str = format!(
        r#"
        SELECT
            id, uuid, user_id, location_id, name, description, device_type,
            model, firmware_version, mac_address, sensor_type, actuator_type, device_scale,
            adopted_at, operation_status, last_seen_at, ip_address, publish_qos, subscribe_qos,
            status_retain, data_retain, lwt_enabled, lwt_message, lwt_qos, lwt_retain,
            heartbeat_interval, offline_threshold, last_command, last_command_at,
            is_active, created_at, updated_at
        FROM devices
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
        where_clause
    );

    let mut query = sqlx::query_as::<_, Device>(&query_str)
        .bind(user_id)
        .bind(limit)
        .bind(offset);

    if location_bind {
        if let Some(ref loc_uuid) = filter.location_uuid {
            query = query.bind(loc_uuid);
        }
    }

    let items = query.fetch_all(pool).await.map_err(|e| {
        error!(error = %e, "fn: device_list_query");
        map_device_db_error(&e)
    })?;

    Ok(items)
}

#[instrument(skip(pool), fields(user_id = user_id, filter = ?filter))]
pub async fn device_count_query(
    user_id: i64,
    filter: &DeviceFilter,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    let mut conditions = vec!["user_id = ?1".to_string()];

    match filter.is_active {
        IsActiveFilter::Active => conditions.push("is_active = 1".to_string()),
        IsActiveFilter::Inactive => conditions.push("is_active = 0".to_string()),
        IsActiveFilter::All => {}
    }

    match filter.operation_status {
        OperationStatusFilter::Online => conditions.push("operation_status = 'online'".to_string()),
        OperationStatusFilter::Offline => conditions.push("operation_status = 'offline'".to_string()),
        OperationStatusFilter::All => {}
    }

    match filter.device_type {
        DeviceTypeFilter::Actuator => conditions.push("device_type = 'actuator'".to_string()),
        DeviceTypeFilter::Sensor => conditions.push("device_type = 'sensor'".to_string()),
        DeviceTypeFilter::All => {}
    }

    let location_bind = if filter.location_uuid.is_some() {
        conditions.push(
            "location_id = (SELECT id FROM locations WHERE uuid = ?2 AND user_id = ?1)".to_string()
        );
        true
    } else {
        false
    };

    let where_clause = conditions.join(" AND ");

    let query_str = format!(
        "SELECT COUNT(*) as total FROM devices WHERE {}",
        where_clause
    );

    let mut query = sqlx::query_as::<_, (i64,)>(&query_str).bind(user_id);

    if location_bind {
        if let Some(ref loc_uuid) = filter.location_uuid {
            query = query.bind(loc_uuid);
        }
    }

    let (total,) = query.fetch_one(pool).await.map_err(|e| {
        error!(error = %e, "fn: device_count_query");
        map_device_db_error(&e)
    })?;

    Ok(total)
}

#[instrument(skip(pool), fields(user_id = user_id, device_uuid = %device_uuid))]
pub async fn device_get_by_uuid_query(
    user_id: i64,
    device_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<Device, String> {
    let rec = sqlx::query_as::<_, Device>(
        r#"
        SELECT
            id, uuid, user_id, location_id, name, description, device_type,
            model, firmware_version, mac_address, sensor_type, actuator_type, device_scale,
            adopted_at, operation_status, last_seen_at, ip_address, publish_qos, subscribe_qos,
            status_retain, data_retain, lwt_enabled, lwt_message, lwt_qos, lwt_retain,
            heartbeat_interval, offline_threshold, last_command, last_command_at,
            is_active, created_at, updated_at
        FROM devices
        WHERE uuid = ?1 AND user_id = ?2
        "#,
    )
    .bind(device_uuid)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Device not found".to_string();
        }
        error!(error = %e, "fn: device_get_by_uuid_query");
        map_device_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool, update_data), fields(user_id = user_id, device_uuid = %device_uuid))]
pub async fn device_update_query(
    user_id: i64,
    device_uuid: &str,
    update_data: &DeviceUpdateDB,
    pool: &Pool<Sqlite>,
) -> Result<Device, String> {
    let mut set_clauses = Vec::new();
    let mut bind_index = 1;

    if update_data.name.is_some() {
        set_clauses.push(format!("name = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.description.is_some() {
        set_clauses.push(format!("description = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.location_id.is_some() {
        set_clauses.push(format!("location_id = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.publish_qos.is_some() {
        set_clauses.push(format!("publish_qos = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.subscribe_qos.is_some() {
        set_clauses.push(format!("subscribe_qos = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.status_retain.is_some() {
        set_clauses.push(format!("status_retain = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.data_retain.is_some() {
        set_clauses.push(format!("data_retain = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.lwt_enabled.is_some() {
        set_clauses.push(format!("lwt_enabled = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.lwt_qos.is_some() {
        set_clauses.push(format!("lwt_qos = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.lwt_retain.is_some() {
        set_clauses.push(format!("lwt_retain = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.heartbeat_interval.is_some() {
        set_clauses.push(format!("heartbeat_interval = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.offline_threshold.is_some() {
        set_clauses.push(format!("offline_threshold = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.is_active.is_some() {
        set_clauses.push(format!("is_active = ?{}", bind_index));
        bind_index += 1;
    }

    if set_clauses.is_empty() {
        return Err("No fields to update".to_string());
    }

    let set_clause = set_clauses.join(", ");
    let uuid_bind = bind_index;
    let user_id_bind = bind_index + 1;

    let query_str = format!(
        r#"
        UPDATE devices
        SET {}
        WHERE uuid = ?{} AND user_id = ?{}
        RETURNING
            id, uuid, user_id, location_id, name, description, device_type,
            model, firmware_version, mac_address, sensor_type, actuator_type, device_scale,
            adopted_at, operation_status, last_seen_at, ip_address, publish_qos, subscribe_qos,
            status_retain, data_retain, lwt_enabled, lwt_message, lwt_qos, lwt_retain,
            heartbeat_interval, offline_threshold, last_command, last_command_at,
            is_active, created_at, updated_at
        "#,
        set_clause, uuid_bind, user_id_bind
    );

    let mut query_builder = sqlx::query_as::<_, Device>(&query_str);

    if let Some(ref name) = update_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref description) = update_data.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(location_id) = update_data.location_id {
        query_builder = query_builder.bind(location_id);
    }
    if let Some(publish_qos) = update_data.publish_qos {
        query_builder = query_builder.bind(publish_qos);
    }
    if let Some(subscribe_qos) = update_data.subscribe_qos {
        query_builder = query_builder.bind(subscribe_qos);
    }
    if let Some(status_retain) = update_data.status_retain {
        query_builder = query_builder.bind(status_retain);
    }
    if let Some(data_retain) = update_data.data_retain {
        query_builder = query_builder.bind(data_retain);
    }
    if let Some(lwt_enabled) = update_data.lwt_enabled {
        query_builder = query_builder.bind(lwt_enabled);
    }
    if let Some(lwt_qos) = update_data.lwt_qos {
        query_builder = query_builder.bind(lwt_qos);
    }
    if let Some(lwt_retain) = update_data.lwt_retain {
        query_builder = query_builder.bind(lwt_retain);
    }
    if let Some(heartbeat_interval) = update_data.heartbeat_interval {
        query_builder = query_builder.bind(heartbeat_interval);
    }
    if let Some(offline_threshold) = update_data.offline_threshold {
        query_builder = query_builder.bind(offline_threshold);
    }
    if let Some(is_active) = update_data.is_active {
        query_builder = query_builder.bind(is_active);
    }

    query_builder = query_builder.bind(device_uuid).bind(user_id);

    let rec = query_builder.fetch_one(pool).await.map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Device not found".to_string();
        }
        if let SqlxError::Database(db_err) = &e {
            if db_err.code().as_deref() == Some("2067") {
                error!(code = ?db_err.code(), message = %db_err, "device_update_query: unique constraint");
                return "Device name already exists".to_string();
            }
        }
        error!(error = %e, "fn: device_update_query");
        map_device_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(location_id = location_id))]
pub async fn get_location_uuid_by_id(
    location_id: i64,
    pool: &Pool<Sqlite>,
) -> Result<String, String> {
    let (uuid,): (String,) = sqlx::query_as("SELECT uuid FROM locations WHERE id = ?1")
        .bind(location_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: get_location_uuid_by_id");
            map_device_db_error(&e)
        })?;

    Ok(uuid)
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn get_user_uuid_by_id(user_id: i64, pool: &Pool<Sqlite>) -> Result<String, String> {
    let (uuid,): (String,) = sqlx::query_as("SELECT uuid FROM users WHERE id = ?1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: get_user_uuid_by_id");
            map_device_db_error(&e)
        })?;

    Ok(uuid)
}

#[instrument(skip(pool), fields(device_id = device_id, command = %command, source = %source))]
pub async fn device_command_insert_query(
    device_id: i64,
    command: &str,
    source: &str,
    pool: &Pool<Sqlite>,
) -> Result<DeviceCommand, String> {
    let rec = sqlx::query_as::<_, DeviceCommand>(
        r#"
        INSERT INTO device_commands (device_id, command, source)
        VALUES (?1, ?2, ?3)
        RETURNING id, device_id, command, source, sent_at, ack_at, response_ms
        "#,
    )
    .bind(device_id)
    .bind(command)
    .bind(source)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_command_insert_query");
        map_device_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn device_update_last_command_query(
    device_id: i64,
    command: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE devices
        SET last_command = ?1, last_command_at = CURRENT_TIMESTAMP
        WHERE id = ?2
        "#,
    )
    .bind(command)
    .bind(device_id)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_update_last_command_query");
        map_device_db_error(&e)
    })?;

    Ok(())
}

#[instrument(skip(pool, filter), fields(device_id = device_id, page = page, page_size = page_size))]
pub async fn device_commands_list_query(
    device_id: i64,
    page: u32,
    page_size: u32,
    filter: &DeviceCommandFilter,
    pool: &Pool<Sqlite>,
) -> Result<Vec<DeviceCommand>, String> {
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    let mut conditions = vec!["device_id = ?1".to_string()];
    let mut bind_idx = 4; // After device_id, limit, offset

    if filter.start_date.is_some() {
        conditions.push(format!("sent_at >= ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.end_date.is_some() {
        conditions.push(format!("sent_at <= ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.command.is_some() {
        conditions.push(format!("command = ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.source.is_some() {
        conditions.push(format!("source = ?{}", bind_idx));
    }

    let where_clause = conditions.join(" AND ");

    let query_str = format!(
        r#"
        SELECT id, device_id, command, source, sent_at, ack_at, response_ms
        FROM device_commands
        WHERE {}
        ORDER BY sent_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
        where_clause
    );

    let mut query = sqlx::query_as::<_, DeviceCommand>(&query_str)
        .bind(device_id)
        .bind(limit)
        .bind(offset);

    if let Some(ref start) = filter.start_date {
        query = query.bind(start);
    }
    if let Some(ref end) = filter.end_date {
        query = query.bind(end);
    }
    if let Some(ref cmd) = filter.command {
        query = query.bind(cmd);
    }
    if let Some(ref src) = filter.source {
        query = query.bind(src);
    }

    let items = query.fetch_all(pool).await.map_err(|e| {
        error!(error = %e, "fn: device_commands_list_query");
        map_device_db_error(&e)
    })?;

    Ok(items)
}

#[instrument(skip(pool, filter), fields(device_id = device_id))]
pub async fn device_commands_count_query(
    device_id: i64,
    filter: &DeviceCommandFilter,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    let mut conditions = vec!["device_id = ?1".to_string()];
    let mut bind_idx = 2;

    if filter.start_date.is_some() {
        conditions.push(format!("sent_at >= ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.end_date.is_some() {
        conditions.push(format!("sent_at <= ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.command.is_some() {
        conditions.push(format!("command = ?{}", bind_idx));
        bind_idx += 1;
    }
    if filter.source.is_some() {
        conditions.push(format!("source = ?{}", bind_idx));
    }

    let where_clause = conditions.join(" AND ");

    let query_str = format!(
        "SELECT COUNT(*) FROM device_commands WHERE {}",
        where_clause
    );

    let mut query = sqlx::query_as::<_, (i64,)>(&query_str).bind(device_id);

    if let Some(ref start) = filter.start_date {
        query = query.bind(start);
    }
    if let Some(ref end) = filter.end_date {
        query = query.bind(end);
    }
    if let Some(ref cmd) = filter.command {
        query = query.bind(cmd);
    }
    if let Some(ref src) = filter.source {
        query = query.bind(src);
    }

    let (total,) = query.fetch_one(pool).await.map_err(|e| {
        error!(error = %e, "fn: device_commands_count_query");
        map_device_db_error(&e)
    })?;

    Ok(total)
}

#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn device_commands_daily_stats_query(
    device_id: i64,
    start_date: &str,
    end_date: &str,
    pool: &Pool<Sqlite>,
) -> Result<Vec<DeviceCommandDailyStats>, String> {
    let items = sqlx::query_as::<_, DeviceCommandDailyStats>(
        r#"
        SELECT 
            DATE(sent_at) as date,
            command,
            COUNT(*) as count,
            AVG(response_ms) as avg_response_ms
        FROM device_commands
        WHERE device_id = ?1 AND sent_at >= ?2 AND sent_at <= ?3
        GROUP BY DATE(sent_at), command
        ORDER BY date
        "#,
    )
    .bind(device_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_commands_daily_stats_query");
        map_device_db_error(&e)
    })?;

    Ok(items)
}

#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn device_commands_summary_query(
    device_id: i64,
    start_date: &str,
    end_date: &str,
    pool: &Pool<Sqlite>,
) -> Result<DeviceCommandSummary, String> {
    let summary = sqlx::query_as::<_, DeviceCommandSummary>(
        r#"
        SELECT 
            COUNT(*) as total_commands,
            COUNT(CASE WHEN command = 'ON' THEN 1 END) as on_count,
            COUNT(CASE WHEN command = 'OFF' THEN 1 END) as off_count,
            AVG(response_ms) as avg_response_ms,
            COUNT(CASE WHEN ack_at IS NULL THEN 1 END) as failed_count
        FROM device_commands
        WHERE device_id = ?1 AND sent_at >= ?2 AND sent_at <= ?3
        "#,
    )
    .bind(device_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_commands_summary_query");
        map_device_db_error(&e)
    })?;

    Ok(summary)
}

#[instrument(skip(pool), fields(device_id = device_id, start_date = %start_date, end_date = %end_date))]
pub async fn device_commands_for_chart_query(
    device_id: i64,
    start_date: &str,
    end_date: &str,
    limit: Option<i64>,
    pool: &Pool<Sqlite>,
) -> Result<Vec<DeviceCommandChartPoint>, String> {
    let limit_val = limit.unwrap_or(1000);

    let items = sqlx::query_as::<_, DeviceCommandChartPoint>(
        r#"
        SELECT command, sent_at, source
        FROM device_commands
        WHERE device_id = ?1 AND sent_at >= ?2 AND sent_at <= ?3
        ORDER BY sent_at ASC
        LIMIT ?4
        "#,
    )
    .bind(device_id)
    .bind(start_date)
    .bind(end_date)
    .bind(limit_val)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: device_commands_for_chart_query");
        map_device_db_error(&e)
    })?;

    Ok(items)
}
