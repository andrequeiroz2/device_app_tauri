use sqlx::Pool;
use sqlx::Sqlite;
use tracing::{error, info, instrument};

use crate::api::auth::auth_validator::validate_bearer;
use crate::api::device::device_query::device_get_by_uuid_query;
use crate::api::model::{ApiError, ApiResponse};
use crate::api::trigger::trigger_model::{
    TriggerCreateInput, TriggerDeleteInput, TriggerListParams, TriggerListResponse,
    TriggerPublic, TriggerUpdateInput,
};
use crate::api::trigger::trigger_query::{
    trigger_delete_query, trigger_get_by_uuid_query, trigger_insert_query, trigger_list_query,
    trigger_update_query,
};
use crate::api::trigger::trigger_notifier::{
    format_trigger_notification_message, send_discord, send_telegram,
    TriggerNotificationContent,
};
use crate::api::trigger::trigger_validator::{
    validate_action_device_command_against_spec, validate_condition_value_in_device_range,
};
use crate::api::user::user_query::user_get_by_uuid_query;
use crate::collector::persistence::query::{
    get_device_name_by_id_query, get_location_name_by_device_id_query,
};

#[instrument(skip(token, input, pool))]
pub async fn create_trigger_handler(
    token: &str,
    input: &TriggerCreateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<TriggerPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    input.validate().map_err(ApiError::err)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_trigger_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = match &input.device_uuid {
        Some(du) if !du.trim().is_empty() => {
            let device =
                device_get_by_uuid_query(user.id, du.trim(), pool)
                    .await
                    .map_err(ApiError::err)?;
            Some(device.id)
        }
        _ => None,
    };

    if input.source_event == "sensor_reading" {
        let du = input
            .device_uuid
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::err("sensor_reading trigger requires a device".to_string()))?;
        let device = device_get_by_uuid_query(user.id, du, pool)
            .await
            .map_err(ApiError::err)?;
        validate_condition_value_in_device_range(
            device.parameter_ranges.as_deref(),
            &input.condition_json,
        )
        .map_err(ApiError::err)?;
    }

    if input.action_type == "device_command" {
        let target_uuid = input
            .action_config_json
            .get("target_device_uuid")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ApiError::err("action_config_json: target_device_uuid required".to_string()))?;
        let target = device_get_by_uuid_query(user.id, target_uuid, pool)
            .await
            .map_err(ApiError::err)?;
        validate_action_device_command_against_spec(
            target.command_spec.as_deref(),
            &input.action_config_json,
        )
        .map_err(ApiError::err)?;
    }

    let db_payload = input.to_db(user.id, device_id);

    let trigger = trigger_insert_query(&db_payload, pool)
        .await
        .map_err(ApiError::err)?;

    let device_uuid = input
        .device_uuid
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    let public = trigger_to_public(&trigger, device_uuid);

    info!(uuid = %public.uuid, "create_trigger_handler: trigger created");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, params, pool))]
pub async fn list_triggers_handler(
    token: &str,
    params: &TriggerListParams,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<TriggerListResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_triggers_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 50);

    let resp = trigger_list_query(user.id, page, page_size, &params.filter, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(resp))
}

#[instrument(skip(token, trigger_uuid, pool))]
pub async fn get_trigger_handler(
    token: &str,
    trigger_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<TriggerPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_trigger_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let (trigger, device_uuid) =
        trigger_get_by_uuid_query(user.id, trigger_uuid, pool)
            .await
            .map_err(ApiError::err)?;

    let public = trigger_to_public(&trigger, device_uuid);

    info!(uuid = %public.uuid, "get_trigger_handler: trigger retrieved");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, input, pool))]
pub async fn update_trigger_handler(
    token: &str,
    input: &TriggerUpdateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<TriggerPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    input.validate().map_err(ApiError::err)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("update_trigger_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let (current_trigger, current_device_uuid) =
        trigger_get_by_uuid_query(user.id, &input.uuid, pool)
            .await
            .map_err(ApiError::err)?;

    let mut device_id_update: Option<Option<i64>> = None;
    let mut effective_device_uuid: Option<String> = None;
    if let Some(du_opt) = &input.device_uuid {
        let resolved = match du_opt {
            Some(du) if !du.trim().is_empty() => {
                let device = device_get_by_uuid_query(user.id, du.trim(), pool)
                    .await
                    .map_err(ApiError::err)?;
                effective_device_uuid = Some(device.uuid.clone());
                Some(device.id)
            }
            _ => None,
        };
        device_id_update = Some(resolved);
    } else {
        effective_device_uuid = current_device_uuid;
    }

    let effective_source_event = input
        .source_event
        .as_deref()
        .unwrap_or(&current_trigger.source_event);
    let effective_condition_json = input.condition_json.as_ref().map(|v| v.clone()).unwrap_or_else(|| {
        serde_json::from_str(&current_trigger.condition_json).unwrap_or(serde_json::Value::Null)
    });
    let effective_action_type = input
        .action_type
        .as_deref()
        .unwrap_or(&current_trigger.action_type);
    let effective_action_config_json = input.action_config_json.as_ref().map(|v| v.clone()).unwrap_or_else(|| {
        serde_json::from_str(&current_trigger.action_config_json).unwrap_or(serde_json::Value::Null)
    });

    if effective_source_event == "sensor_reading" {
        let du = effective_device_uuid
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::err("sensor_reading trigger requires a device".to_string()))?;
        let device = device_get_by_uuid_query(user.id, du, pool)
            .await
            .map_err(ApiError::err)?;
        validate_condition_value_in_device_range(
            device.parameter_ranges.as_deref(),
            &effective_condition_json,
        )
        .map_err(ApiError::err)?;
    }

    if effective_action_type == "device_command" {
        let target_uuid = effective_action_config_json
            .get("target_device_uuid")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ApiError::err("action_config_json: target_device_uuid required".to_string()))?;
        let target = device_get_by_uuid_query(user.id, target_uuid, pool)
            .await
            .map_err(ApiError::err)?;
        validate_action_device_command_against_spec(
            target.command_spec.as_deref(),
            &effective_action_config_json,
        )
        .map_err(ApiError::err)?;
    }

    let mut update_db = crate::api::trigger::trigger_model::TriggerUpdateDB::default();
    update_db.device_id = device_id_update;
    update_db.name = input.name.as_ref().map(|n| n.trim().to_string());
    update_db.source_event = input.source_event.clone();
    update_db.condition_json = input
        .condition_json
        .as_ref()
        .map(|v| v.to_string());
    update_db.action_type = input.action_type.clone();
    update_db.action_config_json = input
        .action_config_json
        .as_ref()
        .map(|v| v.to_string());
    update_db.is_active = input.is_active;
    update_db.cooldown_seconds = input.cooldown_seconds;

    trigger_update_query(user.id, &input.uuid, &update_db, pool)
        .await
        .map_err(ApiError::err)?;

    let (trigger, device_uuid) =
        trigger_get_by_uuid_query(user.id, &input.uuid, pool)
            .await
            .map_err(ApiError::err)?;

    let public = trigger_to_public(&trigger, device_uuid);

    info!(uuid = %public.uuid, "update_trigger_handler: trigger updated");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, payload, pool))]
pub async fn delete_trigger_handler(
    token: &str,
    payload: &TriggerDeleteInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_trigger_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    trigger_delete_query(user.id, &payload.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(()))
}

/// Sends a test notification for a trigger (Discord or Telegram only).
#[instrument(skip(token, pool))]
pub async fn trigger_send_test_handler(
    token: &str,
    trigger_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        error!("trigger_send_test_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let (trigger, _device_uuid) = trigger_get_by_uuid_query(user.id, trigger_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let (device_name, location_name) = if let Some(device_id) = trigger.device_id {
        let device_name = get_device_name_by_id_query(pool, device_id)
            .await
            .map_err(ApiError::err)?
            .unwrap_or_else(|| "Unknown device".to_string());

        let location_name = get_location_name_by_device_id_query(pool, device_id)
            .await
            .map_err(ApiError::err)?;

        (device_name, location_name)
    } else {
        ("Test device".to_string(), None)
    };

    let config: serde_json::Value =
        serde_json::from_str(&trigger.action_config_json).unwrap_or(serde_json::Value::Null);
    let config_obj = config
        .as_object()
        .ok_or_else(|| ApiError::err("Invalid action config".to_string()))?;

    let content = TriggerNotificationContent {
        device_name: device_name.as_str(),
        location_name: location_name.as_deref(),
        subject: "test",
        value: "This is a test notification",
        timestamp: &chrono::Utc::now().to_rfc3339(),
        trigger_name: Some(&trigger.name),
    };
    let severity = config_obj
        .get("severity")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("inf");
    let msg = format_trigger_notification_message(&content, severity);

    match trigger.action_type.as_str() {
        "discord" => {
            let webhook_url = config_obj
                .get("webhook_url")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ApiError::err("Discord webhook_url missing".to_string()))?;
            send_discord(webhook_url, &msg)
                .await
                .map_err(ApiError::err)?;
        }
        "telegram" => {
            let bot_token = config_obj
                .get("bot_token")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ApiError::err("Telegram bot_token missing".to_string()))?;
            let chat_id = config_obj
                .get("chat_id")
                .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_i64().map(|n| n.to_string())))
                .ok_or_else(|| ApiError::err("Telegram chat_id missing".to_string()))?;
            send_telegram(bot_token, &chat_id, &msg)
                .await
                .map_err(ApiError::err)?;
        }
        _ => return Err(ApiError::err("Test only supported for Discord and Telegram triggers".to_string())),
    }

    info!(trigger_uuid = %trigger.uuid, "trigger_send_test_handler: test sent");
    Ok(ApiResponse::ok(()))
}

fn trigger_to_public(
    trigger: &crate::api::trigger::trigger_model::Trigger,
    device_uuid: Option<String>,
) -> TriggerPublic {
    let condition_json: serde_json::Value =
        serde_json::from_str(&trigger.condition_json).unwrap_or(serde_json::Value::Null);
    let action_config_json: serde_json::Value =
        serde_json::from_str(&trigger.action_config_json).unwrap_or(serde_json::Value::Null);

    TriggerPublic {
        uuid: trigger.uuid.clone(),
        device_uuid,
        name: trigger.name.clone(),
        source_event: trigger.source_event.clone(),
        condition_json,
        action_type: trigger.action_type.clone(),
        action_config_json,
        is_active: trigger.is_active,
        cooldown_seconds: trigger.cooldown_seconds,
        created_at: trigger.created_at.clone(),
        updated_at: trigger.updated_at.clone(),
    }
}
