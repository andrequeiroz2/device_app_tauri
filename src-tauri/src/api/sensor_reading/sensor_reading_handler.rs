use sqlx::{Pool, Sqlite};
use tracing::{error, info, instrument};

use crate::api::auth::auth_validator::validate_bearer;
use crate::api::model::{ApiError, ApiResponse};
use crate::api::user::user_query::user_get_by_uuid_query;

use super::sensor_reading_model::{
    AggregationPeriod, SensorReading, SensorReadingAggregated, SensorReadingAggregatedFilter,
    SensorReadingBatchInput, SensorReadingCreateDB, SensorReadingCreateInput, SensorReadingFilter,
    SensorReadingLatest, SensorReadingPublic,
};
use super::sensor_reading_query::{
    sensor_reading_aggregated_query, sensor_reading_batch_insert, sensor_reading_count_query,
    sensor_reading_delete_old_query, sensor_reading_insert, sensor_reading_latest_all_query,
    sensor_reading_latest_query, sensor_reading_list_query,
};

async fn get_device_id_by_uuid(device_uuid: &str, pool: &Pool<Sqlite>) -> Result<i64, String> {
    let result: Option<(i64,)> = sqlx::query_as(
        r#"SELECT id FROM devices WHERE uuid = ?1 AND is_active = 1"#,
    )
    .bind(device_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    result
        .map(|(id,)| id)
        .ok_or_else(|| "Device not found".to_string())
}

async fn get_device_uuid_by_id(device_id: i64, pool: &Pool<Sqlite>) -> Result<String, String> {
    let result: Option<(String,)> = sqlx::query_as(
        r#"SELECT uuid FROM devices WHERE id = ?1"#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    result
        .map(|(uuid,)| uuid)
        .ok_or_else(|| "Device not found".to_string())
}

fn reading_to_public(
    reading: &SensorReading,
    device_uuid: String,
) -> SensorReadingPublic {
    SensorReadingPublic {
        device_uuid,
        measurement: reading.measurement.clone(),
        value: reading.value,
        scale: reading.scale.clone(),
        recorded_at: reading.recorded_at.clone(),
        received_at: reading.received_at.clone(),
    }
}

#[instrument(skip(token, input, pool), fields(device_uuid = %input.device_uuid, measurement = %input.measurement))]
pub async fn create_sensor_reading_handler(
    token: &str,
    input: &SensorReadingCreateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<SensorReadingPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_sensor_reading_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(&input.device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let db_input = SensorReadingCreateDB {
        device_id,
        measurement: input.measurement.clone(),
        value: input.value,
        scale: input.scale.clone(),
        recorded_at: input.recorded_at.clone(),
    };

    let reading = sensor_reading_insert(&db_input, pool)
        .await
        .map_err(ApiError::err)?;

    info!(
        "Sensor reading created: device={}, measurement={}",
        input.device_uuid, input.measurement
    );

    Ok(ApiResponse::ok(reading_to_public(
        &reading,
        input.device_uuid.clone(),
    )))
}

#[instrument(skip(token, input, pool), fields(device_uuid = %input.device_uuid, count = input.readings.len()))]
pub async fn create_sensor_reading_batch_handler(
    token: &str,
    input: &SensorReadingBatchInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<i64>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_sensor_reading_batch_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(&input.device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let readings: Vec<(String, f64, String, String)> = input
        .readings
        .iter()
        .map(|r| {
            (
                r.measurement.clone(),
                r.value,
                r.scale.clone(),
                r.recorded_at.clone(),
            )
        })
        .collect();

    let inserted = sensor_reading_batch_insert(device_id, &readings, pool)
        .await
        .map_err(ApiError::err)?;

    info!(
        "Batch sensor readings created: device={}, count={}",
        input.device_uuid, inserted
    );

    Ok(ApiResponse::ok(inserted))
}

#[instrument(skip(token, filter, pool))]
pub async fn list_sensor_readings_handler(
    token: &str,
    filter: &SensorReadingFilter,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Vec<SensorReadingPublic>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_sensor_readings_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_uuid = filter
        .device_uuid
        .as_ref()
        .ok_or_else(|| ApiError::err("device_uuid is required".to_string()))?;

    let device_id = get_device_id_by_uuid(device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let readings = sensor_reading_list_query(
        device_id,
        filter.measurement.as_deref(),
        filter.start_date.as_deref(),
        filter.end_date.as_deref(),
        filter.limit,
        filter.offset,
        pool,
    )
    .await
    .map_err(ApiError::err)?;

    let public: Vec<SensorReadingPublic> = readings
        .iter()
        .map(|r| reading_to_public(r, device_uuid.clone()))
        .collect();

    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, pool), fields(device_uuid = %device_uuid, measurement = %measurement))]
pub async fn get_sensor_reading_latest_handler(
    token: &str,
    device_uuid: &str,
    measurement: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Option<SensorReadingLatest>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_sensor_reading_latest_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let latest = sensor_reading_latest_query(device_id, measurement, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(latest))
}

#[instrument(skip(token, pool), fields(device_uuid = %device_uuid))]
pub async fn get_sensor_reading_latest_all_handler(
    token: &str,
    device_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Vec<SensorReadingLatest>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_sensor_reading_latest_all_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let latest = sensor_reading_latest_all_query(device_id, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(latest))
}

#[instrument(skip(token, filter, pool))]
pub async fn get_sensor_reading_aggregated_handler(
    token: &str,
    filter: &SensorReadingAggregatedFilter,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Vec<SensorReadingAggregated>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_sensor_reading_aggregated_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(&filter.device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let period = filter.period.unwrap_or(AggregationPeriod::Hour);

    let aggregated = sensor_reading_aggregated_query(
        device_id,
        &filter.measurement,
        &filter.start_date,
        &filter.end_date,
        period,
        pool,
    )
    .await
    .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(aggregated))
}

#[instrument(skip(token, pool), fields(device_uuid = %device_uuid))]
pub async fn get_sensor_reading_count_handler(
    token: &str,
    device_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<i64>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_sensor_reading_count_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let count = sensor_reading_count_query(device_id, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(count))
}

#[instrument(skip(token, pool), fields(device_uuid = %device_uuid, before_date = %before_date))]
pub async fn delete_sensor_reading_old_handler(
    token: &str,
    device_uuid: &str,
    before_date: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<u64>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_sensor_reading_old_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device_id = get_device_id_by_uuid(device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let deleted = sensor_reading_delete_old_query(device_id, before_date, pool)
        .await
        .map_err(ApiError::err)?;

    info!(
        "Old sensor readings deleted: device={}, before={}, count={}",
        device_uuid, before_date, deleted
    );

    Ok(ApiResponse::ok(deleted))
}
