use actix_web::{web, HttpResponse, Result as ActixResult};
use sqlx::Pool;
use sqlx::Sqlite;
use tracing::{info, error};
use crate::api::model::{ApiResponse, ApiError};
use crate::collector::persistence::query::{
    get_mqtt_messages_query,
    count_mqtt_messages_query,
    get_last_message_at_query,
    get_default_broker_status_query,
};
use crate::collector::model::{CollectorStatus, PublishMessageInput};

/// GET /api/status
/// Returns collector status and MQTT connection status
pub async fn get_status(pool: web::Data<Pool<Sqlite>>) -> ActixResult<HttpResponse> {
    info!("GET /api/status");

    // Get broker status
    let (broker_name, mqtt_connected) = match get_default_broker_status_query(&pool).await {
        Ok(Some((name, connected))) => (Some(name), connected),
        Ok(None) => (None, false),
        Err(e) => {
            error!(error = %e, "Failed to get broker status");
            return Ok(HttpResponse::InternalServerError().json(ApiError::err(e)));
        }
    };

    // Get message statistics
    let total_messages = count_mqtt_messages_query(&pool).await
        .unwrap_or(0);
    
    let last_message_at = get_last_message_at_query(&pool).await
        .ok()
        .flatten();

    let status = CollectorStatus {
        running: true,
        mqtt_connected,
        broker_name,
        last_message_at,
        total_messages,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::ok(status)))
}

/// GET /api/messages?topic=...&limit=...
/// Returns MQTT messages history
pub async fn get_messages(
    pool: web::Data<Pool<Sqlite>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    info!("GET /api/messages");

    let topic = query.get("topic").map(|s| s.as_str());
    let limit = query.get("limit")
        .and_then(|s| s.parse::<i64>().ok());

    match get_mqtt_messages_query(topic, limit, &pool).await {
        Ok(messages) => {
            Ok(HttpResponse::Ok().json(ApiResponse::ok(messages)))
        }
        Err(e) => {
            error!(error = %e, "Failed to get messages");
            Ok(HttpResponse::InternalServerError().json(ApiError::err(e)))
        }
    }
}

/// POST /api/publish
/// Publishes a message to MQTT broker
pub async fn publish_message(
    _pool: web::Data<Pool<Sqlite>>,
    payload: web::Json<PublishMessageInput>,
) -> ActixResult<HttpResponse> {
    info!(topic = %payload.topic, "POST /api/publish");

    // TODO: Implement MQTT publish
    // For now, just return success
    // This will be implemented when we have access to the MQTT client
    
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

