//! Trigger evaluation and action execution (Fase 3: run triggers on sensor_reading and device_command).

use serde_json::Value;
use sqlx::{Pool, Sqlite};
use tracing::{error, instrument, warn};

use crate::api::trigger::trigger_evaluator::{
    evaluate_device_command_condition, evaluate_sensor_reading_condition,
};
use crate::api::trigger::trigger_executor::{device_command_payload_from_config, execute_device_command};
use crate::api::trigger::trigger_model::Trigger;
use crate::api::trigger::trigger_notifier::{
    format_trigger_notification_message, send_discord, send_telegram, TriggerNotificationContent,
};
use crate::api::trigger::trigger_query::triggers_list_active_by_device_and_source_query;
use crate::api::trigger::trigger_query::try_acquire_notification_cooldown_query;
use crate::collector::persistence::query::{get_device_name_by_id_query, get_location_name_by_device_id_query};
use crate::collector::state::CollectorState;

/// (measurement, value, scale, recorded_at) - same as SensorReadingTuple in data_processor.
pub type ReadingTuple = (String, f64, String, String);

/// After sensor_reading_batch_insert: load triggers, evaluate condition per reading, run actions (non-blocking).
#[instrument(skip(pool, collector_state, readings), fields(device_id = device_id, n_readings = readings.len()))]
pub async fn run_sensor_reading_triggers(
    device_id: i64,
    readings: &[ReadingTuple],
    pool: &Pool<Sqlite>,
    collector_state: &CollectorState,
) {
    let device_name = match get_device_name_by_id_query(pool, device_id).await {
        Ok(Some(name)) => name,
        Ok(None) => {
            warn!(device_id = device_id, "Device not found for trigger evaluation");
            return;
        }
        Err(e) => {
            error!(error = %e, "get_device_name_by_id for triggers");
            return;
        }
    };

    let location_name = match get_location_name_by_device_id_query(pool, device_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!(
                device_id = device_id,
                error = %e,
                "Failed to fetch location name for trigger notification"
            );
            None
        }
    };

    let triggers = match triggers_list_active_by_device_and_source_query(
        device_id,
        "sensor_reading",
        pool,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "triggers_list_active_by_device_and_source");
            return;
        }
    };
    if triggers.is_empty() {
        return;
    }

    for (measurement, value, _scale, recorded_at) in readings {
        let value_str = format!("{}", value);
        let timestamp = recorded_at.as_str();

        for trigger in &triggers {
            let cond: Value = match serde_json::from_str(&trigger.condition_json) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !evaluate_sensor_reading_condition(&cond, measurement, *value) {
                continue;
            }
            let content = TriggerNotificationContent {
                device_name: &device_name,
                location_name: location_name.as_deref(),
                subject: measurement,
                value: &value_str,
                timestamp,
                trigger_name: Some(&trigger.name),
            };
            run_trigger_action(
                trigger,
                &content,
                None::<&Value>,
                pool,
                collector_state,
            )
            .await;
        }
    }
}

/// After sending a device command: load triggers, evaluate condition, run actions.
#[instrument(skip(pool, collector_state), fields(device_id = device_id, user_id = user_id))]
pub async fn run_device_command_triggers(
    device_id: i64,
    device_name: &str,
    user_id: i64,
    command_payload_json: &Value,
    pool: &Pool<Sqlite>,
    collector_state: &CollectorState,
) {
    let triggers = match triggers_list_active_by_device_and_source_query(
        device_id,
        "device_command",
        pool,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "triggers_list_active_by_device_and_source device_command");
            return;
        }
    };

    let value_str = command_payload_json
        .get("action")
        .and_then(Value::as_str)
        .map(String::from)
        .unwrap_or_else(|| command_payload_json.to_string());
    let timestamp = chrono::Utc::now().to_rfc3339();

    let location_name = match get_location_name_by_device_id_query(pool, device_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!(
                device_id = device_id,
                error = %e,
                "Failed to fetch location name for trigger notification"
            );
            None
        }
    };

    for trigger in &triggers {
        let cond: Value = match serde_json::from_str(&trigger.condition_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !evaluate_device_command_condition(&cond, command_payload_json) {
            continue;
        }
        let content = TriggerNotificationContent {
            device_name,
            location_name: location_name.as_deref(),
            subject: "command",
            value: &value_str,
            timestamp: &timestamp,
            trigger_name: Some(&trigger.name),
        };
        run_trigger_action(
            trigger,
            &content,
            Some(command_payload_json),
            pool,
            collector_state,
        )
        .await;
    }
}

/// Executes a single trigger action (discord, telegram, device_command). Logs errors, does not panic.
async fn run_trigger_action(
    trigger: &Trigger,
    content: &TriggerNotificationContent<'_>,
    _command_payload: Option<&Value>,
    pool: &Pool<Sqlite>,
    collector_state: &CollectorState,
) {
    let action_config: Value = match serde_json::from_str(&trigger.action_config_json) {
        Ok(v) => v,
        Err(e) => {
            error!(trigger_uuid = %trigger.uuid, error = %e, "Invalid action_config_json");
            return;
        }
    };
    let config_obj = match action_config.as_object() {
        Some(o) => o,
        None => {
            error!(trigger_uuid = %trigger.uuid, "action_config_json is not an object");
            return;
        }
    };

    let severity = match config_obj.get("severity") {
        None => "inf",
        Some(v) => match v.as_str() {
            Some(s) => match s {
                "inf" | "att" | "warn" | "critical" => s,
                _ => {
                    warn!(
                        trigger_uuid = %trigger.uuid,
                        invalid_severity = s,
                        "Invalid severity in action_config_json; falling back to 'inf'"
                    );
                    "inf"
                }
            },
            None => {
                warn!(
                    trigger_uuid = %trigger.uuid,
                    "severity in action_config_json is not a string; falling back to 'inf'"
                );
                "inf"
            }
        },
    };
    let msg = format_trigger_notification_message(content, severity);

    match trigger.action_type.as_str() {
        "discord" => {
            if trigger.cooldown_seconds > 0 {
                let now_iso = chrono::Utc::now().to_rfc3339();
                let acquired = match try_acquire_notification_cooldown_query(
                    pool,
                    &trigger.uuid,
                    &now_iso,
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            trigger_uuid = %trigger.uuid,
                            error = %e,
                            "cooldown acquire (discord) failed"
                        );
                        return;
                    }
                };
                if !acquired {
                    tracing::debug!(
                        trigger_uuid = %trigger.uuid,
                        cooldown_seconds = trigger.cooldown_seconds,
                        severity = severity,
                        "Cooldown blocked discord notification"
                    );
                    return;
                }
            }
            let webhook_url = match config_obj.get("webhook_url").and_then(Value::as_str) {
                Some(u) => u,
                None => {
                    error!(trigger_uuid = %trigger.uuid, "discord: webhook_url missing");
                    return;
                }
            };
            tracing::debug!(
                trigger_uuid = %trigger.uuid,
                severity = severity,
                msg_len = msg.len(),
                "Dispatching discord notification"
            );
            if let Err(e) = send_discord(webhook_url, &msg).await {
                error!(trigger_uuid = %trigger.uuid, error = %e, "send_discord failed");
            }
        }
        "telegram" => {
            if trigger.cooldown_seconds > 0 {
                let now_iso = chrono::Utc::now().to_rfc3339();
                let acquired = match try_acquire_notification_cooldown_query(
                    pool,
                    &trigger.uuid,
                    &now_iso,
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            trigger_uuid = %trigger.uuid,
                            error = %e,
                            "cooldown acquire (telegram) failed"
                        );
                        return;
                    }
                };
                if !acquired {
                    tracing::debug!(
                        trigger_uuid = %trigger.uuid,
                        cooldown_seconds = trigger.cooldown_seconds,
                        severity = severity,
                        "Cooldown blocked telegram notification"
                    );
                    return;
                }
            }
            let bot_token = match config_obj.get("bot_token").and_then(Value::as_str) {
                Some(t) => t,
                None => {
                    error!(trigger_uuid = %trigger.uuid, "telegram: bot_token missing");
                    return;
                }
            };
            let chat_id = match config_obj.get("chat_id") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                Some(_) | None => {
                    error!(trigger_uuid = %trigger.uuid, "telegram: chat_id missing or invalid");
                    return;
                }
            };
            tracing::debug!(
                trigger_uuid = %trigger.uuid,
                severity = severity,
                msg_len = msg.len(),
                "Dispatching telegram notification"
            );
            if let Err(e) = send_telegram(bot_token, &chat_id, &msg).await {
                error!(trigger_uuid = %trigger.uuid, error = %e, "send_telegram failed");
            }
        }
        "device_command" => {
            let target_uuid = match config_obj.get("target_device_uuid").and_then(Value::as_str) {
                Some(u) => u,
                None => {
                    error!(trigger_uuid = %trigger.uuid, "device_command: target_device_uuid missing");
                    return;
                }
            };
            let payload_str = match device_command_payload_from_config(config_obj) {
                Ok(s) => s,
                Err(e) => {
                    error!(trigger_uuid = %trigger.uuid, error = %e, "device_command: payload build failed");
                    return;
                }
            };
            if let Err(e) = execute_device_command(
                collector_state,
                pool,
                trigger.user_id,
                target_uuid,
                &payload_str,
            )
            .await
            {
                error!(trigger_uuid = %trigger.uuid, error = %e, "execute_device_command failed");
            }
        }
        _ => {
            warn!(trigger_uuid = %trigger.uuid, action_type = %trigger.action_type, "Unknown action_type");
        }
    }
}
