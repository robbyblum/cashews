use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use config::Config;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use unicode_normalization::UnicodeNormalization;

pub mod cache;

#[derive(Deserialize)]
pub struct ChronConfig {
    pub database_uri: String,
    pub maps_api_key: Option<String>,

    #[serde(default)]
    pub jitter: bool,
}

pub fn normalize_location(s: &str) -> String {
    s.to_lowercase().nfkc().to_string()
}

pub fn load_config() -> anyhow::Result<ChronConfig> {
    // maybe we shouldn't do this here idk
    // tracing_subscriber::fmt::init();
    tracing_subscriber::fmt().compact().without_time().init();

    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("CHRON"))
        .build()?
        .try_deserialize()?;
    Ok(settings)
}

pub struct ObjectId([u8; 12]);

impl Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FromStr for ObjectId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let engine = base64::engine::general_purpose::URL_SAFE;
        // let data = engine.decode(s)?;
        // if data.len() != 32 {
        //     return Err(anyhow::anyhow!("invalid page token"));
        // }

        // let timestamp_nanos = i128::from_le_bytes(data[0..16].try_into().unwrap());
        // let timestamp = OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos)?;
        // let entity_id = Uuid::from_slice(&data[16..32])?;

        // Ok(PageToken {
        //     entity_id,
        //     timestamp,
        // })
        Ok(todo!())
    }
}

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        ObjectId::from_str(&str).map_err(|_| serde::de::Error::custom("invalid object id"))
    }
}

pub fn objectid_to_timestamp(id: &str) -> anyhow::Result<OffsetDateTime> {
    if id.len() != 24 {
        return Err(anyhow!("not a valid objectid"));
    }

    let mut data = [0u8; 12];
    hex::decode_to_slice(id, &mut data)?;

    let unix_timestamp = u32::from_be_bytes(data[0..4].try_into().unwrap());
    Ok(OffsetDateTime::from_unix_timestamp(unix_timestamp as i64)?)
}
