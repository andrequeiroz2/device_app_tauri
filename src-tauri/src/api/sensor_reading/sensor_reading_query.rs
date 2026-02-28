use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use super::sensor_reading_model::{
    AggregationPeriod, SensorReading, SensorReadingAggregated, SensorReadingCreateDB,
    SensorReadingLatest,
};

#[instrument(skip(pool, reading), fields(device_id = reading.device_id, measurement = %reading.measurement))]
pub async fn sensor_reading_insert(
    reading: &SensorReadingCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<SensorReading, String> {
    let rec = sqlx::query_as::<_, SensorReading>(
        r#"
        INSERT INTO sensor_readings (device_id, measurement, value, scale, recorded_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        RETURNING id, device_id, measurement, value, scale, recorded_at, received_at
        "#,
    )
    .bind(reading.device_id)
    .bind(&reading.measurement)
    .bind(reading.value)
    .bind(&reading.scale)
    .bind(&reading.recorded_at)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_insert");
        format!("Failed to insert sensor reading: {}", e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool, readings), fields(device_id = device_id, count = readings.len()))]
pub async fn sensor_reading_batch_insert(
    device_id: i64,
    readings: &[(String, f64, String, String)],
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    let mut inserted = 0i64;

    for (measurement, value, scale, recorded_at) in readings {
        sqlx::query(
            r#"
            INSERT INTO sensor_readings (device_id, measurement, value, scale, recorded_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(device_id)
        .bind(measurement)
        .bind(value)
        .bind(scale)
        .bind(recorded_at)
        .execute(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: sensor_reading_batch_insert");
            format!("Failed to insert sensor reading: {}", e)
        })?;

        inserted += 1;
    }

    Ok(inserted)
}

#[instrument(skip(pool), fields(device_id = device_id, measurement = ?measurement, start = ?start_date, end = ?end_date))]
pub async fn sensor_reading_list_query(
    device_id: i64,
    measurement: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &Pool<Sqlite>,
) -> Result<Vec<SensorReading>, String> {
    let limit_val = limit.unwrap_or(1000);
    let offset_val = offset.unwrap_or(0);

    let readings = match (measurement, start_date, end_date) {
        (Some(m), Some(start), Some(end)) => {
            sqlx::query_as::<_, SensorReading>(
                r#"
                SELECT id, device_id, measurement, value, scale, recorded_at, received_at
                FROM sensor_readings
                WHERE device_id = ?1 AND measurement = ?2 AND recorded_at >= ?3 AND recorded_at <= ?4
                ORDER BY recorded_at ASC
                LIMIT ?5 OFFSET ?6
                "#,
            )
            .bind(device_id)
            .bind(m)
            .bind(start)
            .bind(end)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(pool)
            .await
        }
        (Some(m), None, None) => {
            sqlx::query_as::<_, SensorReading>(
                r#"
                SELECT id, device_id, measurement, value, scale, recorded_at, received_at
                FROM sensor_readings
                WHERE device_id = ?1 AND measurement = ?2
                ORDER BY recorded_at ASC
                LIMIT ?3 OFFSET ?4
                "#,
            )
            .bind(device_id)
            .bind(m)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(pool)
            .await
        }
        (None, Some(start), Some(end)) => {
            sqlx::query_as::<_, SensorReading>(
                r#"
                SELECT id, device_id, measurement, value, scale, recorded_at, received_at
                FROM sensor_readings
                WHERE device_id = ?1 AND recorded_at >= ?2 AND recorded_at <= ?3
                ORDER BY recorded_at ASC
                LIMIT ?4 OFFSET ?5
                "#,
            )
            .bind(device_id)
            .bind(start)
            .bind(end)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(pool)
            .await
        }
        _ => {
            sqlx::query_as::<_, SensorReading>(
                r#"
                SELECT id, device_id, measurement, value, scale, recorded_at, received_at
                FROM sensor_readings
                WHERE device_id = ?1
                ORDER BY recorded_at ASC
                LIMIT ?2 OFFSET ?3
                "#,
            )
            .bind(device_id)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(pool)
            .await
        }
    };

    readings.map_err(|e| {
        error!(error = %e, "fn: sensor_reading_list_query");
        format!("Failed to list sensor readings: {}", e)
    })
}

#[instrument(skip(pool), fields(device_id = device_id, measurement = %measurement))]
pub async fn sensor_reading_latest_query(
    device_id: i64,
    measurement: &str,
    pool: &Pool<Sqlite>,
) -> Result<Option<SensorReadingLatest>, String> {
    let reading = sqlx::query_as::<_, SensorReading>(
        r#"
        SELECT id, device_id, measurement, value, scale, recorded_at, received_at
        FROM sensor_readings
        WHERE device_id = ?1 AND measurement = ?2
        ORDER BY recorded_at DESC
        LIMIT 1
        "#,
    )
    .bind(device_id)
    .bind(measurement)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_latest_query");
        format!("Failed to get latest reading: {}", e)
    })?;

    Ok(reading.map(|r| SensorReadingLatest {
        measurement: r.measurement,
        value: r.value,
        scale: r.scale,
        recorded_at: r.recorded_at,
    }))
}

#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn sensor_reading_latest_all_query(
    device_id: i64,
    pool: &Pool<Sqlite>,
) -> Result<Vec<SensorReadingLatest>, String> {
    let readings = sqlx::query_as::<_, SensorReading>(
        r#"
        SELECT sr.id, sr.device_id, sr.measurement, sr.value, sr.scale, sr.recorded_at, sr.received_at
        FROM sensor_readings sr
        INNER JOIN (
            SELECT measurement, MAX(recorded_at) as max_recorded
            FROM sensor_readings
            WHERE device_id = ?1
            GROUP BY measurement
        ) latest ON sr.measurement = latest.measurement AND sr.recorded_at = latest.max_recorded
        WHERE sr.device_id = ?1
        "#,
    )
    .bind(device_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_latest_all_query");
        format!("Failed to get latest readings: {}", e)
    })?;

    Ok(readings
        .into_iter()
        .map(|r| SensorReadingLatest {
            measurement: r.measurement,
            value: r.value,
            scale: r.scale,
            recorded_at: r.recorded_at,
        })
        .collect())
}

#[instrument(skip(pool), fields(device_id = device_id, measurement = %measurement, period = ?period))]
pub async fn sensor_reading_aggregated_query(
    device_id: i64,
    measurement: &str,
    start_date: &str,
    end_date: &str,
    period: AggregationPeriod,
    pool: &Pool<Sqlite>,
) -> Result<Vec<SensorReadingAggregated>, String> {
    let date_format = period.to_sqlite_format();

    let readings = sqlx::query_as::<_, SensorReadingAggregated>(&format!(
        r#"
        SELECT 
            strftime('{}', recorded_at) as period,
            AVG(value) as avg_value,
            MIN(value) as min_value,
            MAX(value) as max_value,
            COUNT(*) as count
        FROM sensor_readings
        WHERE device_id = ?1 
          AND measurement = ?2
          AND recorded_at >= ?3
          AND recorded_at <= ?4
        GROUP BY period
        ORDER BY period ASC
        "#,
        date_format
    ))
    .bind(device_id)
    .bind(measurement)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_aggregated_query");
        format!("Failed to aggregate sensor readings: {}", e)
    })?;

    Ok(readings)
}

#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn sensor_reading_count_query(
    device_id: i64,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM sensor_readings WHERE device_id = ?1
        "#,
    )
    .bind(device_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_count_query");
        format!("Failed to count sensor readings: {}", e)
    })?;

    Ok(count.0)
}

#[instrument(skip(pool), fields(device_id = device_id, before_date = %before_date))]
pub async fn sensor_reading_delete_old_query(
    device_id: i64,
    before_date: &str,
    pool: &Pool<Sqlite>,
) -> Result<u64, String> {
    let result = sqlx::query(
        r#"
        DELETE FROM sensor_readings
        WHERE device_id = ?1 AND recorded_at < ?2
        "#,
    )
    .bind(device_id)
    .bind(before_date)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: sensor_reading_delete_old_query");
        format!("Failed to delete old readings: {}", e)
    })?;

    Ok(result.rows_affected())
}
