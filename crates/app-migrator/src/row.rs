use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, FixedOffset}
};

#[derive(Debug, FromRow)]
pub struct MigrationRow {
    pub version: i64,
    pub description: String,
    pub installed_on: DateTime<FixedOffset>,
    pub success: bool,
    pub checksum: Vec<u8>,
    pub execution_time: i64
}
