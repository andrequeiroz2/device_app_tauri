/// Parses MQTT topic to extract broker UUID and device UUID
/// Expected format: `{broker_uuid}/{device_uuid}/...`
/// 
/// # Examples
/// ```
/// let (broker, device) = parse_topic_uuid("abc-123/def-456/status");
/// // broker = Some("abc-123"), device = Some("def-456")
/// ```
pub fn parse_topic_uuid(topic: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = topic.split('/').collect();
    
    if parts.len() >= 2 {
        let broker_uuid = parts[0].to_string();
        let device_uuid = parts[1].to_string();
        
        // Basic validation: UUIDs should not be empty
        if !broker_uuid.is_empty() && !device_uuid.is_empty() {
            return (Some(broker_uuid), Some(device_uuid));
        }
    }
    
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_topic_uuid() {
        let (broker, device) = parse_topic_uuid("abc-123/def-456/status");
        assert_eq!(broker, Some("abc-123".to_string()));
        assert_eq!(device, Some("def-456".to_string()));
    }

    #[test]
    fn test_parse_topic_uuid_with_data() {
        let (broker, device) = parse_topic_uuid("broker-uuid/device-uuid/data/temperature");
        assert_eq!(broker, Some("broker-uuid".to_string()));
        assert_eq!(device, Some("device-uuid".to_string()));
    }

    #[test]
    fn test_parse_topic_uuid_invalid() {
        let (broker, device) = parse_topic_uuid("invalid");
        assert_eq!(broker, None);
        assert_eq!(device, None);
    }
}

