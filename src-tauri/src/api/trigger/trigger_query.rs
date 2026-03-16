use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::error::map_trigger_db_error;
use crate::api::trigger::trigger_model::{
    Trigger, TriggerCreateDB, TriggerFilter, TriggerListResponse, TriggerPublic,
    TriggerUpdateDB, TriggerWithDeviceRow,
};

#[instrument(skip(trigger, pool), fields(uuid = %trigger.uuid, user_id = trigger.user_id, name = %trigger.name))]
pub async fn trigger_insert_query(
    trigger: &TriggerCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<Trigger, String> {
    let rec = sqlx::query_as::<_, Trigger>(
        r#"
        INSERT INTO triggers (
            uuid, user_id, device_id, name, source_event,
            condition_json, action_type, action_config_json, is_active
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        RETURNING
            id, uuid, user_id, device_id, name, source_event,
            condition_json, action_type, action_config_json, is_active,
            created_at, updated_at
        "#,
    )
    .bind(&trigger.uuid)
    .bind(trigger.user_id)
    .bind(trigger.device_id)
    .bind(&trigger.name)
    .bind(&trigger.source_event)
    .bind(&trigger.condition_json)
    .bind(&trigger.action_type)
    .bind(&trigger.action_config_json)
    .bind(trigger.is_active)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: trigger_insert_query");
        map_trigger_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id, trigger_uuid = %trigger_uuid))]
pub async fn trigger_delete_query(
    user_id: i64,
    trigger_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"DELETE FROM triggers WHERE user_id = ?1 AND uuid = ?2"#,
    )
    .bind(user_id)
    .bind(trigger_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: trigger_delete_query");
        map_trigger_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Trigger not found".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, page = page, page_size = page_size, filter = ?filter))]
pub async fn trigger_list_query(
    user_id: i64,
    page: u32,
    page_size: u32,
    filter: &TriggerFilter,
    pool: &Pool<Sqlite>,
) -> Result<TriggerListResponse, String> {
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    let mut conditions = vec!["t.user_id = ?1".to_string()];
    let mut bind_device: Option<String> = None;

    if let Some(ref du) = filter.device_uuid {
        conditions.push("d.uuid = ?2".to_string());
        bind_device = Some(du.clone());
    }
    if filter.is_active.is_some() {
        let idx = if bind_device.is_some() { 3 } else { 2 };
        conditions.push(format!("t.is_active = ?{}", idx));
    }

    let where_clause = conditions.join(" AND ");
    let has_device_filter = bind_device.is_some();

    let sql = if has_device_filter {
        format!(
            r#"
            SELECT
                t.id, t.uuid, t.user_id, t.device_id, t.name, t.source_event,
                t.condition_json, t.action_type, t.action_config_json, t.is_active,
                t.created_at, t.updated_at,
                d.uuid as device_uuid
            FROM triggers t
            LEFT JOIN devices d ON t.device_id = d.id
            WHERE {}
            ORDER BY t.created_at DESC
            LIMIT ?{} OFFSET ?{}
            "#,
            where_clause,
            if filter.is_active.is_some() { 4 } else { 3 },
            if filter.is_active.is_some() { 5 } else { 4 },
        )
    } else {
        format!(
            r#"
            SELECT
                t.id, t.uuid, t.user_id, t.device_id, t.name, t.source_event,
                t.condition_json, t.action_type, t.action_config_json, t.is_active,
                t.created_at, t.updated_at,
                d.uuid as device_uuid
            FROM triggers t
            LEFT JOIN devices d ON t.device_id = d.id
            WHERE {}
            ORDER BY t.created_at DESC
            LIMIT ?{} OFFSET ?{}
            "#,
            where_clause,
            if filter.is_active.is_some() { 3 } else { 2 },
            if filter.is_active.is_some() { 4 } else { 3 },
        )
    };

    let mut query_builder = sqlx::query_as::<_, TriggerWithDeviceRow>(&sql).bind(user_id);
    if let Some(ref du) = bind_device {
        query_builder = query_builder.bind(du);
    }
    if let Some(is_active) = filter.is_active {
        query_builder = query_builder.bind(is_active);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(pool).await.map_err(|e| {
        error!(error = %e, "fn: trigger_list_query");
        map_trigger_db_error(&e)
    })?;

    let items: Vec<TriggerPublic> = rows
        .into_iter()
        .map(|r| {
            let condition_json: serde_json::Value = serde_json::from_str(&r.condition_json)
                .unwrap_or(serde_json::Value::Null);
            let action_config_json: serde_json::Value =
                serde_json::from_str(&r.action_config_json).unwrap_or(serde_json::Value::Null);
            TriggerPublic {
                uuid: r.uuid,
                device_uuid: r.device_uuid,
                name: r.name,
                source_event: r.source_event,
                condition_json,
                action_type: r.action_type,
                action_config_json,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        })
        .collect();

    let total = trigger_count_query(user_id, filter, pool).await?;

    Ok(TriggerListResponse {
        items,
        total,
        page,
        page_size,
    })
}

/// Lists active triggers for a device and source_event (for evaluation in collector / send_command).
#[instrument(skip(pool), fields(device_id = ?device_id, source_event = %source_event))]
pub async fn triggers_list_active_by_device_and_source_query(
    device_id: i64,
    source_event: &str,
    pool: &Pool<Sqlite>,
) -> Result<Vec<Trigger>, String> {
    let rows = sqlx::query_as::<_, Trigger>(
        r#"
        SELECT id, uuid, user_id, device_id, name, source_event,
               condition_json, action_type, action_config_json, is_active,
               created_at, updated_at
        FROM triggers
        WHERE device_id = ?1 AND source_event = ?2 AND is_active = 1
        ORDER BY id
        "#,
    )
    .bind(device_id)
    .bind(source_event)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: triggers_list_active_by_device_and_source_query");
        map_trigger_db_error(&e)
    })?;

    Ok(rows)
}

#[instrument(skip(pool), fields(user_id = user_id, filter = ?filter))]
pub async fn trigger_count_query(
    user_id: i64,
    filter: &TriggerFilter,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    let mut conditions = vec!["user_id = ?1".to_string()];
    let mut bind_device: Option<String> = None;

    if let Some(ref du) = filter.device_uuid {
        conditions.push("device_id = (SELECT id FROM devices WHERE uuid = ?2 AND user_id = ?1)".to_string());
        bind_device = Some(du.clone());
    }
    if filter.is_active.is_some() {
        let idx = if bind_device.is_some() { 3 } else { 2 };
        conditions.push(format!("is_active = ?{}", idx));
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT COUNT(*) as total FROM triggers WHERE {}",
        where_clause
    );

    let mut query_builder = sqlx::query_as::<_, (i64,)>(&sql).bind(user_id);
    if let Some(ref du) = bind_device {
        query_builder = query_builder.bind(du);
    }
    if let Some(is_active) = filter.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let (total,) = query_builder.fetch_one(pool).await.map_err(|e| {
        error!(error = %e, "fn: trigger_count_query");
        map_trigger_db_error(&e)
    })?;

    Ok(total)
}

#[instrument(skip(pool), fields(user_id = user_id, trigger_uuid = %trigger_uuid))]
pub async fn trigger_get_by_uuid_query(
    user_id: i64,
    trigger_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(Trigger, Option<String>), String> {
    let rec = sqlx::query_as::<_, TriggerWithDeviceRow>(
        r#"
        SELECT
            t.id, t.uuid, t.user_id, t.device_id, t.name, t.source_event,
            t.condition_json, t.action_type, t.action_config_json, t.is_active,
            t.created_at, t.updated_at,
            d.uuid as device_uuid
        FROM triggers t
        LEFT JOIN devices d ON t.device_id = d.id
        WHERE t.uuid = ?1 AND t.user_id = ?2
        "#,
    )
    .bind(trigger_uuid)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: trigger_get_by_uuid_query");
        map_trigger_db_error(&e)
    })?;

    let r = rec.ok_or("Trigger not found".to_string())?;

    let trigger = Trigger {
        id: r.id,
        uuid: r.uuid,
        user_id: r.user_id,
        device_id: r.device_id,
        name: r.name,
        source_event: r.source_event,
        condition_json: r.condition_json,
        action_type: r.action_type,
        action_config_json: r.action_config_json,
        is_active: r.is_active,
        created_at: r.created_at,
        updated_at: r.updated_at,
    };

    Ok((trigger, r.device_uuid))
}

#[instrument(skip(pool, update_data), fields(user_id = user_id, trigger_uuid = %trigger_uuid))]
pub async fn trigger_update_query(
    user_id: i64,
    trigger_uuid: &str,
    update_data: &TriggerUpdateDB,
    pool: &Pool<Sqlite>,
) -> Result<Trigger, String> {
    let mut set_clauses = Vec::new();
    let mut bind_index = 1;

    if update_data.device_id.is_some() {
        set_clauses.push(format!("device_id = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.name.is_some() {
        set_clauses.push(format!("name = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.source_event.is_some() {
        set_clauses.push(format!("source_event = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.condition_json.is_some() {
        set_clauses.push(format!("condition_json = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.action_type.is_some() {
        set_clauses.push(format!("action_type = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.action_config_json.is_some() {
        set_clauses.push(format!("action_config_json = ?{}", bind_index));
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
        UPDATE triggers
        SET {}
        WHERE uuid = ?{} AND user_id = ?{}
        RETURNING
            id, uuid, user_id, device_id, name, source_event,
            condition_json, action_type, action_config_json, is_active,
            created_at, updated_at
        "#,
        set_clause, uuid_bind, user_id_bind
    );

    let mut query_builder = sqlx::query_as::<_, Trigger>(&query_str);

    if let Some(did) = update_data.device_id {
        query_builder = query_builder.bind(did);
    }
    if let Some(ref name) = update_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref se) = update_data.source_event {
        query_builder = query_builder.bind(se);
    }
    if let Some(ref cj) = update_data.condition_json {
        query_builder = query_builder.bind(cj);
    }
    if let Some(ref at) = update_data.action_type {
        query_builder = query_builder.bind(at);
    }
    if let Some(ref acj) = update_data.action_config_json {
        query_builder = query_builder.bind(acj);
    }
    if let Some(active) = update_data.is_active {
        query_builder = query_builder.bind(active);
    }

    query_builder = query_builder.bind(trigger_uuid).bind(user_id);

    let rec = query_builder.fetch_optional(pool).await.map_err(|e| {
        error!(error = %e, "fn: trigger_update_query");
        map_trigger_db_error(&e)
    })?;

    rec.ok_or("Trigger not found".to_string())
}
