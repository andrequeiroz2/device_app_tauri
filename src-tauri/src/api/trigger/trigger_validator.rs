use serde_json::Value;
use tracing::instrument;

const VALID_OPERATORS: &[&str] = &[">=", "<=", "==", "!=", ">", "<"];
const DISCORD_WEBHOOK_PREFIX: &str = "https://discord.com/api/webhooks/";

/// Valida condition_json conforme source_event.
#[instrument(skip(condition))]
pub fn validate_condition_json(source_event: &str, condition: &Value) -> Result<(), String> {
    if condition.is_null() {
        return Err("condition_json is required".to_string());
    }

    let obj = condition.as_object().ok_or("condition_json must be an object")?;

    match source_event {
        "sensor_reading" => validate_sensor_reading_condition(obj),
        "device_command" => validate_device_command_condition(obj),
        "schedule" => validate_schedule_condition(obj),
        _ => Err(format!("Invalid source_event: {}", source_event)),
    }
}

fn validate_sensor_reading_condition(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let measurement = obj
        .get("measurement")
        .and_then(Value::as_str)
        .ok_or("condition_json: measurement (string) is required for sensor_reading")?;
    if measurement.trim().is_empty() {
        return Err("condition_json: measurement cannot be empty".to_string());
    }

    let operator = obj
        .get("operator")
        .and_then(Value::as_str)
        .ok_or("condition_json: operator is required for sensor_reading")?;
    if !VALID_OPERATORS.contains(&operator) {
        return Err(format!(
            "condition_json: operator must be one of: {}",
            VALID_OPERATORS.join(", ")
        ));
    }

    let value = obj.get("value").ok_or("condition_json: value is required for sensor_reading")?;
    if !matches!(value, Value::Number(_)) {
        return Err("condition_json: value must be a number for sensor_reading".to_string());
    }

    Ok(())
}

fn validate_device_command_condition(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let has_command = obj.contains_key("command");
    let has_pattern = obj.contains_key("command_pattern");

    if has_command && has_pattern {
        return Err("condition_json: use either command or command_pattern, not both".to_string());
    }
    if !has_command && !has_pattern {
        return Err("condition_json: command or command_pattern is required for device_command".to_string());
    }

    if has_command {
        let cmd = obj.get("command").and_then(Value::as_str).ok_or(
            "condition_json: command must be a string",
        )?;
        if cmd.trim().is_empty() {
            return Err("condition_json: command cannot be empty".to_string());
        }
    }

    if has_pattern {
        let _pat = obj.get("command_pattern").ok_or("condition_json: command_pattern is required")?;
        if !_pat.is_object() {
            return Err("condition_json: command_pattern must be an object".to_string());
        }
    }

    Ok(())
}

fn validate_schedule_condition(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let days = obj
        .get("days_of_week")
        .and_then(Value::as_array)
        .ok_or("condition_json: days_of_week (array) is required for schedule")?;
    for d in days {
        let n = d.as_i64().ok_or("condition_json: days_of_week must contain numbers 0-6")?;
        if !(0..=6).contains(&n) {
            return Err("condition_json: days_of_week values must be 0 (Sun) to 6 (Sat)".to_string());
        }
    }

    let time = obj
        .get("time")
        .and_then(Value::as_str)
        .ok_or("condition_json: time (HH:mm) is required for schedule")?;
    if !is_valid_time(time) {
        return Err("condition_json: time must be in HH:mm format".to_string());
    }

    let start = obj
        .get("start_date")
        .and_then(Value::as_str)
        .ok_or("condition_json: start_date (YYYY-MM-DD) is required for schedule")?;
    if !is_valid_date(start) {
        return Err("condition_json: start_date must be YYYY-MM-DD".to_string());
    }

    let end = obj
        .get("end_date")
        .and_then(Value::as_str)
        .ok_or("condition_json: end_date (YYYY-MM-DD) is required for schedule")?;
    if !is_valid_date(end) {
        return Err("condition_json: end_date must be YYYY-MM-DD".to_string());
    }

    Ok(())
}

fn is_valid_time(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let h: Result<u8, _> = parts[0].parse();
    let m: Result<u8, _> = parts[1].parse();
    match (h, m) {
        (Ok(h), Ok(m)) => h <= 23 && m <= 59,
        _ => false,
    }
}

fn is_valid_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let y: Result<i32, _> = parts[0].parse();
    let m: Result<u8, _> = parts[1].parse();
    let d: Result<u8, _> = parts[2].parse();
    match (y, m, d) {
        (Ok(_y), Ok(m), Ok(d)) => (1..=12).contains(&m) && (1..=31).contains(&d),
        _ => false,
    }
}

/// Valida action_config_json conforme action_type.
#[instrument(skip(config))]
pub fn validate_action_config_json(action_type: &str, config: &Value) -> Result<(), String> {
    if config.is_null() {
        return Err("action_config_json is required".to_string());
    }

    let obj = config
        .as_object()
        .ok_or("action_config_json must be an object")?;

    match action_type {
        "discord" => validate_discord_config(obj),
        "telegram" => validate_telegram_config(obj),
        "device_command" => validate_device_command_config(obj),
        _ => Err(format!("Invalid action_type: {}", action_type)),
    }
}

fn validate_discord_config(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let url = obj
        .get("webhook_url")
        .and_then(Value::as_str)
        .ok_or("action_config_json: webhook_url is required for discord")?;
    if url.trim().is_empty() {
        return Err("action_config_json: webhook_url cannot be empty".to_string());
    }
    if !url.starts_with(DISCORD_WEBHOOK_PREFIX) {
        return Err(format!(
            "action_config_json: webhook_url must start with {}",
            DISCORD_WEBHOOK_PREFIX
        ));
    }
    validate_optional_severity(obj)
}

fn validate_telegram_config(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let token = obj
        .get("bot_token")
        .and_then(Value::as_str)
        .ok_or("action_config_json: bot_token is required for telegram")?;
    if token.trim().is_empty() {
        return Err("action_config_json: bot_token cannot be empty".to_string());
    }

    let chat_id = obj.get("chat_id").ok_or(
        "action_config_json: chat_id is required for telegram",
    )?;
    if chat_id.is_string() {
        let s = chat_id.as_str().unwrap_or("");
        if s.trim().is_empty() {
            return Err("action_config_json: chat_id cannot be empty".to_string());
        }
    } else if !chat_id.is_number() {
        return Err("action_config_json: chat_id must be string or number".to_string());
    }

    validate_optional_severity(obj)
}

fn validate_optional_severity(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    if let Some(sev_val) = obj.get("severity") {
        let sev = sev_val
            .as_str()
            .ok_or("action_config_json: severity must be a string")?;
        match sev {
            "inf" | "att" | "warn" | "critical" => Ok(()),
            _ => Err(format!(
                "action_config_json: invalid severity '{}'",
                sev
            )),
        }
    } else {
        Ok(())
    }
}

fn validate_device_command_config(obj: &serde_json::Map<String, Value>) -> Result<(), String> {
    let target = obj
        .get("target_device_uuid")
        .and_then(Value::as_str)
        .ok_or("action_config_json: target_device_uuid is required for device_command action")?;
    if target.trim().is_empty() {
        return Err("action_config_json: target_device_uuid cannot be empty".to_string());
    }

    let has_command = obj.contains_key("command");
    let has_payload = obj.contains_key("command_payload");

    if has_command && has_payload {
        return Err("action_config_json: use either command or command_payload, not both".to_string());
    }
    if !has_command && !has_payload {
        return Err("action_config_json: command or command_payload is required for device_command action".to_string());
    }

    if has_command {
        let cmd = obj.get("command").and_then(Value::as_str).ok_or(
            "action_config_json: command must be a string",
        )?;
        if cmd.trim().is_empty() {
            return Err("action_config_json: command cannot be empty".to_string());
        }
    }

    if has_payload {
        let _pl = obj.get("command_payload").ok_or("action_config_json: command_payload required")?;
        if !_pl.is_object() {
            return Err("action_config_json: command_payload must be an object".to_string());
        }
    }

    Ok(())
}

/// Validates that the condition value (sensor_reading) is within the device's parameter_ranges for the given measurement.
/// Returns Err if device has no parameter_ranges or value is outside [min_reading, max_reading].
#[instrument(skip(parameter_ranges_json, condition))]
pub fn validate_condition_value_in_device_range(
    parameter_ranges_json: Option<&str>,
    condition: &Value,
) -> Result<(), String> {
    let pr = parameter_ranges_json
        .filter(|s| !s.trim().is_empty())
        .ok_or("Device has no parameter_ranges; cannot validate condition value".to_string())?;
    let ranges = serde_json::from_str::<serde_json::Map<String, Value>>(pr)
        .map_err(|e| format!("Device parameter_ranges invalid: {}", e))?;

    let obj = condition
        .as_object()
        .ok_or("condition_json must be an object")?;
    let measurement = obj
        .get("measurement")
        .and_then(Value::as_str)
        .ok_or("condition_json: measurement is required")?;
    let value = obj
        .get("value")
        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
        .ok_or("condition_json: value must be a number")?;

    let entry = ranges
        .get(measurement)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Device has no parameter range for measurement '{}'", measurement))?;
    let min_r = entry
        .get("min_reading")
        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
        .ok_or_else(|| format!("parameter_ranges.{}: min_reading required", measurement))?;
    let max_r = entry
        .get("max_reading")
        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
        .ok_or_else(|| format!("parameter_ranges.{}: max_reading required", measurement))?;

    if value < min_r {
        return Err(format!(
            "Condition value {} is below device range for {} (min_reading: {})",
            value, measurement, min_r
        ));
    }
    if value > max_r {
        return Err(format!(
            "Condition value {} is above device range for {} (max_reading: {})",
            value, measurement, max_r
        ));
    }
    Ok(())
}

/// Validates that the device_command action (command or command_payload value) is within the target device's command_spec.
#[instrument(skip(command_spec_json, action_config))]
pub fn validate_action_device_command_against_spec(
    command_spec_json: Option<&str>,
    action_config: &Value,
) -> Result<(), String> {
    let cs = command_spec_json
        .filter(|s| !s.trim().is_empty())
        .ok_or("Target device has no command_spec; cannot validate command".to_string())?;
    let spec = serde_json::from_str::<serde_json::Map<String, Value>>(cs)
        .map_err(|e| format!("Target device command_spec invalid: {}", e))?;

    let obj = action_config
        .as_object()
        .ok_or("action_config_json must be an object")?;

    let spec_type = spec
        .get("type")
        .and_then(Value::as_str)
        .ok_or("command_spec: type (discrete or range) is required")?;

    if spec_type == "discrete" {
        let commands = spec
            .get("commands")
            .and_then(Value::as_array)
            .ok_or("command_spec: commands array is required for type discrete")?;
        let allowed: Vec<&str> = commands
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        let cmd = obj
            .get("command")
            .and_then(Value::as_str)
            .ok_or("action_config_json: command is required for discrete command_spec")?;
        if !allowed.contains(&cmd) {
            return Err(format!(
                "Command '{}' is not in target device's allowed commands: {:?}",
                cmd, allowed
            ));
        }
        return Ok(());
    }

    if spec_type == "range" {
        let min_v = spec
            .get("min")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
            .ok_or("command_spec: min is required for type range")?;
        let max_v = spec
            .get("max")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
            .ok_or("command_spec: max is required for type range")?;
        let pl = obj
            .get("command_payload")
            .and_then(Value::as_object)
            .ok_or("action_config_json: command_payload is required for range command_spec")?;
        let value = pl
            .get("value")
            .or_else(|| pl.get("angle"))
            .or_else(|| pl.get("position"))
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
            .ok_or("command_payload must contain a numeric value (e.g. value, angle) within device range")?;
        if value < min_v || value > max_v {
            return Err(format!(
                "Command value {} is outside target device range (min: {}, max: {})",
                value, min_v, max_v
            ));
        }
        return Ok(());
    }

    Err(format!("command_spec: unknown type '{}'", spec_type))
}
