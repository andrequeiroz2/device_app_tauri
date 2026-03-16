//! Runtime evaluation of trigger conditions (sensor_reading, device_command).

use serde_json::Value;
use tracing::instrument;

/// Returns true if the sensor reading (measurement, value) satisfies the condition.
#[instrument(skip(condition_json))]
pub fn evaluate_sensor_reading_condition(
    condition_json: &Value,
    measurement: &str,
    value: f64,
) -> bool {
    let obj = match condition_json.as_object() {
        Some(o) => o,
        None => return false,
    };
    let cond_measurement = match obj.get("measurement").and_then(Value::as_str) {
        Some(m) => m,
        None => return false,
    };
    if cond_measurement != measurement {
        return false;
    }
    let operator = match obj.get("operator").and_then(Value::as_str) {
        Some(op) => op,
        None => return false,
    };
    let cond_value = match obj.get("value").and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64))) {
        Some(v) => v,
        None => return false,
    };
    match operator {
        ">=" => value >= cond_value,
        "<=" => value <= cond_value,
        "==" => (value - cond_value).abs() < f64::EPSILON,
        "!=" => (value - cond_value).abs() >= f64::EPSILON,
        ">" => value > cond_value,
        "<" => value < cond_value,
        _ => false,
    }
}

/// Returns true if the command payload satisfies the device_command condition.
/// payload_json: the MQTT command payload as object, e.g. {"action": "ON"} or {"action": "set_temp", "value": 45}.
#[instrument(skip(condition_json, payload_json))]
pub fn evaluate_device_command_condition(condition_json: &Value, payload_json: &Value) -> bool {
    let cond = match condition_json.as_object() {
        Some(o) => o,
        None => return false,
    };
    let payload = match payload_json.as_object() {
        Some(o) => o,
        None => return false,
    };

    if let Some(cmd) = cond.get("command").and_then(Value::as_str) {
        let action = payload.get("action").and_then(Value::as_str);
        return action.map(|a| a == cmd).unwrap_or(false);
    }
    if let Some(pattern) = cond.get("command_pattern").and_then(Value::as_object) {
        for (k, v) in pattern {
            match payload.get(k) {
                Some(pv) if value_eq(v, pv) => continue,
                _ => return false,
            }
        }
        return true;
    }
    false
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            n1.as_f64().unwrap_or(0.0) == n2.as_f64().unwrap_or(0.0)
        }
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sensor_reading_ge() {
        let cond = json!({"measurement": "temperature", "operator": ">=", "value": 80});
        assert!(evaluate_sensor_reading_condition(&cond, "temperature", 85.0));
        assert!(evaluate_sensor_reading_condition(&cond, "temperature", 80.0));
        assert!(!evaluate_sensor_reading_condition(&cond, "temperature", 79.0));
        assert!(!evaluate_sensor_reading_condition(&cond, "humidity", 90.0));
    }

    #[test]
    fn device_command_simple() {
        let cond = json!({"command": "ON"});
        let payload = json!({"action": "ON"});
        assert!(evaluate_device_command_condition(&cond, &payload));
        assert!(!evaluate_device_command_condition(&cond, &json!({"action": "OFF"})));
    }

    #[test]
    fn device_command_pattern() {
        let cond = json!({"command_pattern": {"action": "set_temp", "value": 45}});
        let payload = json!({"action": "set_temp", "value": 45});
        assert!(evaluate_device_command_condition(&cond, &payload));
        assert!(!evaluate_device_command_condition(&cond, &json!({"action": "set_temp", "value": 30})));
    }
}
