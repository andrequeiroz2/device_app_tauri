use tracing::{error, instrument};

const DISCORD_MAX_CONTENT_LEN: usize = 2000;
const TELEGRAM_MAX_TEXT_LEN: usize = 4096;
const HTTP_TIMEOUT_SECS: u64 = 10;

/// Data to build the notification body (Discord/Telegram).
#[derive(Debug, Clone)]
pub struct TriggerNotificationContent<'a> {
    /// Device name (source of the event).
    pub device_name: &'a str,
    /// Optional location name where the device is placed.
    pub location_name: Option<&'a str>,
    /// Event type: measurement (e.g. "temperature", "gas_concentration") or "command" (e.g. "ON", "OFF").
    pub subject: &'a str,
    /// Value: sensor reading or command sent.
    pub value: &'a str,
    /// Event timestamp (e.g. "2025-02-28 10:30:00" or ISO8601).
    pub timestamp: &'a str,
    /// Trigger name (optional), e.g. "Temperature alert".
    pub trigger_name: Option<&'a str>,
}

/// Formats the standard message for trigger notifications.
/// Includes device name, measurement/command, value and timestamp.
pub fn format_trigger_notification_message(
    content: &TriggerNotificationContent<'_>,
    severity: &str,
) -> String {
    let trigger_part = content
        .trigger_name
        .map(|n| format!("[{}] ", n))
        .unwrap_or_default();

    let (icon, label) = match severity {
        "inf" => ("ℹ️", "INFO"),
        "att" => ("❕", "ATTENTION"),
        "warn" => ("🟡", "WARN"),
        "critical" => ("🔴", "CRITICAL"),
        _ => ("ℹ️", "INFO"),
    };

    let location_part = content
        .location_name
        .map(|n| format!(" | Location: {}", n))
        .unwrap_or_default();

    format!(
        "{} {} {}Device: {}{} | {}: {} | {}",
        icon,
        label,
        trigger_part,
        content.device_name,
        location_part,
        content.subject,
        content.value,
        content.timestamp
    )
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .expect("reqwest client build")
}

/// Sends a message to a Discord webhook.
/// Payload: `{"content": "message"}` (up to 2000 characters).
#[instrument(skip(webhook_url, content), fields(url_len = webhook_url.len()))]
pub async fn send_discord(webhook_url: &str, content: &str) -> Result<(), String> {
    let content = truncate_utf8(content, DISCORD_MAX_CONTENT_LEN);

    let payload = serde_json::json!({
        "content": content,
    });

    let client = http_client();
    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "send_discord: request failed");
            format!("Discord request failed: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, body = %body, "send_discord: non-success response");
        return Err(format!(
            "Discord webhook returned {}: {}",
            status,
            body.chars().take(200).collect::<String>()
        ));
    }

    Ok(())
}

/// Sends a message via Telegram Bot API.
/// POST `https://api.telegram.org/bot{token}/sendMessage` with `{"chat_id": "...", "text": "..."}`.
#[instrument(skip(bot_token, text), fields(chat_id = %chat_id))]
pub async fn send_telegram(bot_token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let text = truncate_utf8(text, TELEGRAM_MAX_TEXT_LEN);

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        bot_token.trim()
    );

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
    });

    let client = http_client();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "send_telegram: request failed");
            format!("Telegram request failed: {}", e)
        })?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        error!(status = %status, body = %body, "send_telegram: non-success response");
        return Err(format!(
            "Telegram API returned {}: {}",
            status,
            body.chars().take(200).collect::<String>()
        ));
    }

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
        if parsed.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            let desc = parsed
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            error!(description = %desc, "send_telegram: API ok=false");
            return Err(format!("Telegram API error: {}", desc));
        }
    }

    Ok(())
}

fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_utf8_short() {
        let s = "hello";
        assert_eq!(truncate_utf8(s, 100), "hello");
    }

    #[test]
    fn truncate_utf8_cut() {
        let s = "a".repeat(100);
        let out = truncate_utf8(&s, 10);
        assert!(out.len() <= 10 + 3);
        assert!(out.ends_with("..."));
    }

    #[test]
    fn format_notification_message() {
        let content = TriggerNotificationContent {
            device_name: "Sala",
            location_name: None,
            subject: "temperature",
            value: "85",
            timestamp: "2025-02-28 10:30:00",
            trigger_name: Some("Alerta temperatura"),
        };
        let msg = format_trigger_notification_message(&content, "inf");
        assert!(msg.contains("[Alerta temperatura]"));
        assert!(msg.contains("Device: Sala"));
        assert!(msg.contains("temperature: 85"));
        assert!(msg.contains("2025-02-28 10:30:00"));

        let no_trigger = TriggerNotificationContent {
            device_name: "Cozinha",
            location_name: None,
            subject: "command",
            value: "ON",
            timestamp: "2025-02-28 11:00:00",
            trigger_name: None,
        };
        let msg2 = format_trigger_notification_message(&no_trigger, "inf");
        assert_eq!(
            msg2,
            "ℹ️ INFO Device: Cozinha | command: ON | 2025-02-28 11:00:00"
        );
    }

    /// Mock HTTP: Discord webhook receives POST with JSON body {"content": "..."}.
    #[tokio::test]
    async fn send_discord_mock_http_validates_payload() {
        let mut server = mockito::Server::new_async().await;
        let expected_body = serde_json::json!({ "content": "test alert" });
        let mock = server
            .mock("POST", "/")
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::Json(expected_body))
            .with_status(200)
            .create_async()
            .await;

        let result = send_discord(server.url().as_str(), "test alert").await;
        mock.assert_async().await;
        assert!(result.is_ok(), "send_discord should succeed: {:?}", result);
    }

    /// Mock HTTP: Discord returns 4xx and we return Err.
    #[tokio::test]
    async fn send_discord_mock_http_handles_error_status() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .with_status(404)
            .with_body("Webhook not found")
            .create_async()
            .await;

        let result = send_discord(server.url().as_str(), "test").await;
        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("404"));
    }
}
