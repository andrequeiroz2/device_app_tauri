use sqlx::{Pool, Sqlite};
use tracing::{error, info, instrument};

use crate::api::auth::auth_validator::validate_bearer;
use crate::api::mqtt_broker::mqtt_broker_model::{MqttBrokerCreateInput, MqttBrokerPublic, MqttBrokerListParams, MqttBrokerListResponse, MqttBrokerDeleteInput, MqttBrokerUpdateInput};
use crate::api::mqtt_broker::mqtt_broker_query::{mqtt_broker_post_query, mqtt_broker_unset_other_defaults, mqtt_broker_list_query, mqtt_broker_soft_delete_query, mqtt_broker_get_by_uuid_query, mqtt_broker_update_query};
use crate::api::model::{ApiError, ApiResponse};
use crate::api::user::user_query::user_get_by_uuid_query;

#[instrument(skip(token, input, pool), fields(name = %input.name, host = %input.host, port = ?input.port))]
pub async fn create_mqtt_broker_handler(
    token: &str,
    input: &MqttBrokerCreateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<MqttBrokerPublic>, ApiError> {
    // 1) Auth
    let auth = validate_bearer(token)?;

    // 2) Validate payload
    input.validate().map_err(ApiError::err)?;

    // 3) Resolve user_id by uuid
    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_mqtt_broker_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    // 4) Build DB payload
    let mut db_payload = input.to_db(user.id);

    // 5) Se is_default = true, desmarcar outros defaults do usuário ANTES de inserir
    if db_payload.is_default {
        mqtt_broker_unset_other_defaults(user.id, &db_payload.uuid, pool)
            .await
            .map_err(ApiError::err)?;
        info!("create_mqtt_broker_handler: unset other default brokers");
    }

    // 6) Criptografar password se fornecido
    // TODO: Implementar criptografia reversível (AES-GCM ou ChaCha20Poly1305)
    // Por enquanto, vamos armazenar como está (NÃO SEGURO - apenas para desenvolvimento inicial)
    if let Some(password) = &input.password {
        // TODO: db_payload.password = Some(encrypt_broker_password(password)?);
        db_payload.password = Some(password.clone());
        info!("create_mqtt_broker_handler: password provided (not encrypted yet - TODO)");
    }

    // 7) Insert
    let broker = mqtt_broker_post_query(&db_payload, pool)
        .await
        .map_err(ApiError::err)?;

    // 8) Convert to public
    let public = MqttBrokerPublic::from(broker);

    info!(uuid = %public.uuid, name = %public.name, "create_mqtt_broker_handler: broker created");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, params, pool), fields(page = ?params.page, page_size = ?params.page_size))]
pub async fn list_mqtt_brokers_handler(
    token: &str,
    params: &MqttBrokerListParams,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<MqttBrokerListResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_mqtt_brokers_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 50);

    let resp = mqtt_broker_list_query(user.id, page, page_size, &params.filter, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(resp))
}

#[instrument(skip(token, payload, pool), fields(broker_uuid = %payload.uuid))]
pub async fn delete_mqtt_broker_handler(
    token: &str,
    payload: &MqttBrokerDeleteInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_mqtt_broker_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    mqtt_broker_soft_delete_query(user.id, &payload.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    info!(uuid = %payload.uuid, "delete_mqtt_broker_handler: broker deleted");
    Ok(ApiResponse::ok(()))
}

#[instrument(skip(token, broker_uuid, pool), fields(broker_uuid = %broker_uuid))]
pub async fn get_mqtt_broker_handler(
    token: &str,
    broker_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<MqttBrokerPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_mqtt_broker_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let broker = mqtt_broker_get_by_uuid_query(user.id, broker_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let public = MqttBrokerPublic::from(broker);

    info!(uuid = %public.uuid, "get_mqtt_broker_handler: broker retrieved");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, input, pool), fields(broker_uuid = %input.uuid))]
pub async fn update_mqtt_broker_handler(
    token: &str,
    input: &MqttBrokerUpdateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<MqttBrokerPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    input.validate().map_err(ApiError::err)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("update_mqtt_broker_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let broker = mqtt_broker_update_query(user.id, &input.uuid, input.is_active, pool)
        .await
        .map_err(ApiError::err)?;

    let public = MqttBrokerPublic::from(broker);

    info!(uuid = %public.uuid, "update_mqtt_broker_handler: broker updated");
    Ok(ApiResponse::ok(public))
}

