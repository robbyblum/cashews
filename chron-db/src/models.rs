use std::{fmt::Display, str::FromStr};

use base64::Engine;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::{types::JsonRawValue, FromRow, Type};
use time::{Duration, OffsetDateTime};

#[repr(i16)]
#[derive(Debug, Clone, Copy, Type, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    State = 1,
    League = 2,
    Team = 3,
    Player = 4,
    Game = 5,
    Time = 6,
    Nouns = 7,
    Adjectives = 8,
    PlayerLite = 9,
    TeamLite = 10,
    News = 11,
    Spotlight = 12,
    Election = 13,
    GamesEndpoint = 14,
    PostseasonBracket = 15,
    Message = 16,
    Schedule = 17,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EntityVersion {
    pub kind: EntityKind,
    pub entity_id: String,
    pub valid_from: IsoDateTime,
    pub valid_to: Option<IsoDateTime>,
    pub data: sqlx::types::Json<Box<JsonRawValue>>,
}

impl EntityVersion {
    pub fn parse<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(T::deserialize(&**self.data)?)
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EntityObservation {
    pub kind: EntityKind,
    pub entity_id: String,
    pub timestamp: IsoDateTime,
    pub data: serde_json::Value,
}

impl EntityObservation {
    pub fn parse<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(T::deserialize(&self.data)?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent, no_pg_array)]
pub struct IsoDateTime(#[serde(with = "time::serde::rfc3339")] pub OffsetDateTime);

impl From<OffsetDateTime> for IsoDateTime {
    fn from(value: OffsetDateTime) -> Self {
        IsoDateTime(value)
    }
}

impl Into<OffsetDateTime> for IsoDateTime {
    fn into(self) -> OffsetDateTime {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct PageToken {
    pub entity_id: String,
    pub timestamp: OffsetDateTime,
}

impl Display for PageToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::with_capacity(32);

        let timestamp = (self.timestamp.unix_timestamp_nanos() / 1000) as i64;
        buf.extend_from_slice(&timestamp.to_be_bytes());
        buf.extend_from_slice(self.entity_id.as_bytes());

        let engine = base64::engine::general_purpose::URL_SAFE;
        f.write_str(&engine.encode(&buf))
    }
}

impl FromStr for PageToken {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = base64::engine::general_purpose::URL_SAFE;
        let data = engine.decode(s)?;
        if data.len() <= 16 {
            return Err(anyhow::anyhow!("invalid page token"));
        }

        let timestamp_nanos = i64::from_be_bytes(data[0..8].try_into().unwrap());
        let timestamp =
            OffsetDateTime::from_unix_timestamp_nanos((timestamp_nanos as i128) * 1000)?;
        let entity_id = String::from_utf8_lossy(&data[8..]).into_owned();

        Ok(PageToken {
            entity_id,
            timestamp,
        })
    }
}

impl Serialize for PageToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PageToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        PageToken::from_str(&str).map_err(|_| serde::de::Error::custom("invalid page token"))
    }
}

pub trait HasPageToken {
    fn page_token(&self) -> PageToken;
}

impl HasPageToken for EntityVersion {
    fn page_token(&self) -> PageToken {
        PageToken {
            entity_id: self.entity_id.clone(),
            timestamp: self.valid_from.0,
        }
    }
}

#[derive(Debug)]
pub struct NewObject {
    pub kind: EntityKind,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub timestamp: OffsetDateTime,
    pub request_time: Duration,
}
