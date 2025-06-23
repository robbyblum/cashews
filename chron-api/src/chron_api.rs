use std::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use axum::{
    Json,
    extract::{Query, State},
};
use chron_db::{
    models::{EntityKind, EntityVersion, IsoDateTime, PageToken},
    queries::{PaginatedResult, SortOrder},
};
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};

use crate::{AppError, AppState};

#[derive(Deserialize)]
pub struct GetEntitiesQuery {
    kind: EntityKind,
    at: Option<IsoDateTime>,

    #[serde(deserialize_with = "comma_separated", default)]
    id: Vec<String>,
    #[serde(default)]
    order: SortOrder,
    #[serde(default)]
    count: Option<u64>,
    page: Option<PageToken>,

    before: Option<IsoDateTime>,
    after: Option<IsoDateTime>,
}

pub async fn get_entities(
    State(ctx): State<AppState>,
    Query(q): Query<GetEntitiesQuery>,
) -> Result<Json<PaginatedResult<EntityVersion>>, AppError> {
    let default_count = if q.kind == EntityKind::Game {
        100 // games big  
    } else {
        1000
    };
    let count = q.count.unwrap_or(default_count).min(1000);

    let events = ctx
        .db
        .get_entities(chron_db::queries::GetEntitiesQuery {
            kind: q.kind,
            at: q.at.map(|x| x.0),
            id: q.id,
            order: q.order,
            count: count,
            page: q.page,
            before: q.before.map(|x| x.0),
            after: q.after.map(|x| x.0),
        })
        .await?;

    Ok(Json(events))
}

#[derive(Deserialize, Debug)]
pub struct GetVersionsQuery {
    pub kind: EntityKind,

    #[serde(deserialize_with = "comma_separated", default)]
    pub id: Vec<String>,
    pub before: Option<IsoDateTime>,
    pub after: Option<IsoDateTime>,
    pub count: Option<u64>,
    #[serde(default)]
    pub order: SortOrder,

    pub page: Option<PageToken>,
}

pub async fn get_versions(
    State(ctx): State<AppState>,
    Query(q): Query<GetVersionsQuery>,
) -> Result<Json<PaginatedResult<EntityVersion>>, AppError> {
    let default_count = if q.kind == EntityKind::Game {
        100 // games big  
    } else {
        1000
    };
    let count = q.count.unwrap_or(default_count).min(1000);

    let events = ctx
        .db
        .get_versions(chron_db::queries::GetVersionsQuery {
            kind: q.kind,
            id: q.id,
            before: q.before.map(|x| x.0),
            after: q.after.map(|x| x.0),
            count: count,
            order: q.order,
            page: q.page,
        })
        .await?;

    Ok(Json(events))
}

fn comma_separated<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
where
    V: FromIterator<T>,
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    struct CommaSeparated<V, T>(PhantomData<V>, PhantomData<T>);

    impl<'de, V, T> Visitor<'de> for CommaSeparated<V, T>
    where
        V: FromIterator<T>,
        T: FromStr,
        T::Err: Display,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string containing comma-separated elements")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let iter = s.split(",").map(FromStr::from_str);
            Result::from_iter(iter).map_err(de::Error::custom)
        }
    }

    let visitor = CommaSeparated(PhantomData, PhantomData);
    deserializer.deserialize_str(visitor)
}
