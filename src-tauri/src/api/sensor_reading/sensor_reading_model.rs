use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SensorReading {
    pub id: i64,
    pub device_id: i64,
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
    pub received_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReadingPublic {
    pub device_uuid: String,
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
    pub received_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SensorReadingCreateInput {
    pub device_uuid: String,
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
}

#[derive(Debug)]
pub struct SensorReadingCreateDB {
    pub device_id: i64,
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SensorReadingBatchInput {
    pub device_uuid: String,
    pub readings: Vec<ReadingItem>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingItem {
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct SensorReadingFilter {
    pub device_uuid: Option<String>,
    pub measurement: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReadingLatest {
    pub measurement: String,
    pub value: f64,
    pub scale: String,
    pub recorded_at: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SensorReadingAggregated {
    pub period: Option<String>,
    pub avg_value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AggregationPeriod {
    Hour,
    Day,
}

impl AggregationPeriod {
    pub fn to_sqlite_format(&self) -> &'static str {
        match self {
            AggregationPeriod::Hour => "%Y-%m-%d %H:00",
            AggregationPeriod::Day => "%Y-%m-%d",
        }
    }
}

impl Default for AggregationPeriod {
    fn default() -> Self {
        AggregationPeriod::Hour
    }
}

#[derive(Debug, Deserialize)]
pub struct SensorReadingAggregatedFilter {
    pub device_uuid: String,
    pub measurement: String,
    pub start_date: String,
    pub end_date: String,
    pub period: Option<AggregationPeriod>,
}
